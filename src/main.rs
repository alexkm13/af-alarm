mod wfdb;
mod rr;
mod detector;
mod spsc;
mod af;

use std::sync::Arc;
use std::thread;
use std::thread::Thread;
use spsc::queue::BoundQueue;
use detector::detector;
use std::sync::OnceLock;
use std::sync::atomic::{Ordering, AtomicBool};
use rr::RRProcessor;

fn main() {
    // create queue
    // spawn consumer
    // wire beats through
    // producer thread
    let b_queue = Arc::new(BoundQueue::new(30));
    let complete_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let producer_handle = Arc::new(OnceLock::<Thread>::new());
    let consumer_handle = Arc::new(OnceLock::<Thread>::new());


    let prod_flag = Arc::clone(&complete_flag);

    let prod_handle_slot = Arc::clone(&producer_handle);
    let cons_handle_for_prod = Arc::clone(&consumer_handle);
    
    let prod_queue = Arc::clone(&b_queue);
   
    let producer = thread::spawn(move || {
        prod_handle_slot
            .set(thread::current())
            .expect("producer handle already set");

        let beats = detector("data/mit-bih-arrhythmia-database-1.0.0/108.dat");

        for mut beat in beats {
            loop {
                match prod_queue.push(beat) {
                    Ok(()) => {
                        if let Some(thread) = cons_handle_for_prod.get() {
                            thread.unpark();
                        }
                        break;
                    }
                    Err(returned_beat) => {
                        beat = returned_beat;
                        thread::park();
                    }
                }
            }
        }
        prod_flag.store(true, Ordering::Release);
    });

    let cons_handle_slot = Arc::clone(&consumer_handle);
    let prod_handle_for_cons = Arc::clone(&producer_handle);    
    let cons_queue = Arc::clone(&b_queue);
    let cons_flag = Arc::clone(&complete_flag);

    let consumer = thread::spawn(move || {
        cons_handle_slot
            .set(thread::current())
            .expect("consumer handle already set");

        let mut process: RRProcessor = RRProcessor::new();
        
        'consumer_loop: loop {
            loop {
               match cons_queue.pop() {
                   Ok(beat) => {
                       if let Some(thread) = prod_handle_for_cons.get() {
                           thread.unpark();
                       }

                       process.stream_beat(beat);
                       break;
                   },
                   Err(()) => {
                        if cons_flag.load(Ordering::Acquire) == true && cons_queue.push_count.load(Ordering::Relaxed) == cons_queue.pop_count.load(Ordering::Relaxed) {
                            break 'consumer_loop;
                        } else {
                            thread::park();
                        }
                    }

                }
            }
        }
        process
    });

    producer.join().expect("producer panicked");
    let processor = consumer.join().expect("consumer panicked");

    println!("RR intervals: {}", processor.rr_interval.len());
    println!("Irregularity signals: {}", processor.irregularity.len());

    println!("\n=== Naive Detector (threshold={}) ===", processor.naive.threshold);
    println!("Alarms: {}", processor.naive.alarms.len());

    println!("\n=== CUSUM Detector (k={}, h={}) ===", processor.cusum.k, processor.cusum.h);
    println!("Alarms: {}", processor.cusum.alarms.len());
    println!("Final S_t: {:.2}", processor.cusum.s_t);
}
 

