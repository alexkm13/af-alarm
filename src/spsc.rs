pub mod queue {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::cell::UnsafeCell;
    pub struct BoundQueue {
        pub size: u32,
        pub pop_count: AtomicU32,
        pub push_count: AtomicU32,
        pub q: Vec<UnsafeCell<Option<u32>>>,
    }
    
    unsafe impl Sync for BoundQueue {}

    impl BoundQueue {
        pub fn push(&self, beat: u32) -> Result<(), u32> {
            let curr_size = self.push_count.load(Ordering::Relaxed) - self.pop_count.load(Ordering::Acquire);
            if curr_size < self.size {
                let i = (self.push_count.load(Ordering::Relaxed) as usize) % (self.size as usize);
                unsafe {
                    let ptr = self.q[i].get();
                    *ptr = Some(beat);
                }
                self.push_count.fetch_add(1, Ordering::Release);
                return Ok(());
            } else {
                return Err(beat);
            }
        }

        pub fn pop(&self) -> Result<u32, ()> {
            let curr_size = self.push_count.load(Ordering::Acquire) - self.pop_count.load(Ordering::Relaxed);
            let i = self.pop_count.load(Ordering::Relaxed) as usize % self.size as usize;
            let pt = self.q[i].get();
            
            unsafe {
                if curr_size <= 0 {
                    return Err(());
                } else if *pt != None { 
                    let b = *pt;
                    *pt = None; 
                    self.pop_count.fetch_add(1, Ordering::Release);
                    return Ok(b.unwrap());
                } else {
                    return Err(());
                }
            }
        }
        
        pub fn create_vec(size: u32) -> Vec<UnsafeCell<Option<u32>>> {
            let mut outcome_vec: Vec<UnsafeCell<Option<u32>>> = Vec::new();
            for _ in 0..size {
                outcome_vec.push(UnsafeCell::new(None));
            }
            outcome_vec
        }
        pub fn new(size: u32) -> Self {
            BoundQueue {
                size,
                push_count: AtomicU32::new(0),
                pop_count: AtomicU32::new(0),
                q: Self::create_vec(size),
            }
        }
    }
}   
