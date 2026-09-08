use std::sync::Arc;
use std::thread;
use std::thread::Thread;
use std::sync::OnceLock;
use std::sync::atomic::{Ordering, AtomicBool};
use af_alarm::spsc::queue::BoundQueue;

#[test]
fn pipeline_deterministic_beats() {
    // Known beat locations
    let test_beats: Vec<u32> = vec![100, 450, 800, 1200, 1550];
    // Expected RR intervals: 350, 350, 400, 350
    let expected_rr: Vec<f64> = vec![350.0, 350.0, 400.0, 350.0];

    let b_queue = Arc::new(BoundQueue::new(10));
    let complete_flag = Arc::new(AtomicBool::new(false));
    let producer_handle = Arc::new(OnceLock::<Thread>::new());
    let consumer_handle = Arc::new(OnceLock::<Thread>::new());

    let prod_flag = Arc::clone(&complete_flag);
    let prod_handle_slot = Arc::clone(&producer_handle);
    let cons_handle_for_prod = Arc::clone(&consumer_handle);
    let prod_queue = Arc::clone(&b_queue);

    let beats_for_prod = test_beats.clone();
    let producer = thread::spawn(move || {
        prod_handle_slot
            .set(thread::current())
            .expect("producer handle already set");

        let mut sent_count = 0;
        for mut beat in beats_for_prod {
            loop {
                match prod_queue.push(beat) {
                    Ok(()) => {
                        sent_count += 1;
                        if let Some(t) = cons_handle_for_prod.get() {
                            t.unpark();
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
        // Wake consumer one more time so it can see the flag
        if let Some(t) = cons_handle_for_prod.get() {
            t.unpark();
        }
        sent_count
    });

    let cons_handle_slot = Arc::clone(&consumer_handle);
    let prod_handle_for_cons = Arc::clone(&producer_handle);
    let cons_queue = Arc::clone(&b_queue);
    let cons_flag = Arc::clone(&complete_flag);

    let consumer = thread::spawn(move || {
        cons_handle_slot
            .set(thread::current())
            .expect("consumer handle already set");

        let mut prev_beat: Option<u32> = None;
        let mut rr_list: Vec<(f64, u32)> = Vec::new();
        let mut received: Vec<u32> = Vec::new();

        'consumer_loop: loop {
            loop {
                match cons_queue.pop() {
                    Ok(beat) => {
                        received.push(beat);
                        if let Some(t) = prod_handle_for_cons.get() {
                            t.unpark();
                        }

                        if let Some(prev) = prev_beat {
                            rr_list.push((beat as f64 - prev as f64, beat));
                        }
                        prev_beat = Some(beat);
                        break;
                    }
                    Err(()) => {
                        if cons_flag.load(Ordering::Acquire)
                            && cons_queue.push_count.load(Ordering::Relaxed)
                               == cons_queue.pop_count.load(Ordering::Relaxed)
                        {
                            break 'consumer_loop;
                        } else {
                            thread::park();
                        }
                    }
                }
            }
        }
        (received, rr_list)
    });

    let sent_count = producer.join().expect("producer panicked");
    let (received, rr_list) = consumer.join().expect("consumer panicked");

    // Verify: producer sent every beat
    assert_eq!(sent_count, test_beats.len(), "producer should send all beats");

    // Verify: consumer received them in order
    assert_eq!(received, test_beats, "consumer should receive beats in order");

    // Verify: RR output has exactly beats.len() - 1 intervals
    assert_eq!(rr_list.len(), test_beats.len() - 1, "should have n-1 RR intervals");

    // Verify: every RR tuple is correct
    for (i, (rr, _beat)) in rr_list.iter().enumerate() {
        assert_eq!(*rr, expected_rr[i], "RR interval {} should be {}", i, expected_rr[i]);
    }

    // If we got here, both threads terminated cleanly
    println!("All checks passed!");
}

#[test]
fn pipeline_backpressure() {
    // Queue size 2, send 5 beats - forces producer to park waiting for space
    let test_beats: Vec<u32> = vec![100, 200, 300, 400, 500];

    let b_queue = Arc::new(BoundQueue::new(2)); // Small queue!
    let complete_flag = Arc::new(AtomicBool::new(false));
    let producer_handle = Arc::new(OnceLock::<Thread>::new());
    let consumer_handle = Arc::new(OnceLock::<Thread>::new());

    let prod_flag = Arc::clone(&complete_flag);
    let prod_handle_slot = Arc::clone(&producer_handle);
    let cons_handle_for_prod = Arc::clone(&consumer_handle);
    let prod_queue = Arc::clone(&b_queue);

    let beats_for_prod = test_beats.clone();
    let producer = thread::spawn(move || {
        prod_handle_slot
            .set(thread::current())
            .expect("producer handle already set");

        let mut sent_count = 0;
        let mut park_count = 0;
        for mut beat in beats_for_prod {
            loop {
                match prod_queue.push(beat) {
                    Ok(()) => {
                        sent_count += 1;
                        if let Some(t) = cons_handle_for_prod.get() {
                            t.unpark();
                        }
                        break;
                    }
                    Err(returned_beat) => {
                        park_count += 1;
                        beat = returned_beat;
                        thread::park();
                    }
                }
            }
        }
        prod_flag.store(true, Ordering::Release);
        if let Some(t) = cons_handle_for_prod.get() {
            t.unpark();
        }
        (sent_count, park_count)
    });

    let cons_handle_slot = Arc::clone(&consumer_handle);
    let prod_handle_for_cons = Arc::clone(&producer_handle);
    let cons_queue = Arc::clone(&b_queue);
    let cons_flag = Arc::clone(&complete_flag);

    let consumer = thread::spawn(move || {
        cons_handle_slot
            .set(thread::current())
            .expect("consumer handle already set");

        let mut received: Vec<u32> = Vec::new();

        'consumer_loop: loop {
            loop {
                match cons_queue.pop() {
                    Ok(beat) => {
                        received.push(beat);
                        if let Some(t) = prod_handle_for_cons.get() {
                            t.unpark();
                        }
                        break;
                    }
                    Err(()) => {
                        if cons_flag.load(Ordering::Acquire)
                            && cons_queue.push_count.load(Ordering::Relaxed)
                               == cons_queue.pop_count.load(Ordering::Relaxed)
                        {
                            break 'consumer_loop;
                        } else {
                            thread::park();
                        }
                    }
                }
            }
        }
        received
    });

    let (sent_count, park_count) = producer.join().expect("producer panicked");
    let received = consumer.join().expect("consumer panicked");

    // Verify all beats sent and received
    assert_eq!(sent_count, test_beats.len(), "producer should send all beats");
    assert_eq!(received, test_beats, "consumer should receive beats in order");

    // Producer must have parked at least once (queue size 2, 5 beats)
    assert!(park_count > 0, "producer should have parked due to full queue (parked {} times)", park_count);

    println!("Backpressure test passed! Producer parked {} times", park_count);
}
