pub mod queue {
    use std::collections::VecDeque;
    pub struct BoundQueue {
        size: u32,
        curr_len: u32,
        q: VecDeque<Option<u32>>,
    }

    impl BoundQueue {
        pub fn push(&mut self, beat: u32) -> Result<(), u32> {
            if self.curr_len < self.size {
                self.q.push_front(Some(beat));
                self.curr_len += 1;
                return Ok(());
            } else {
                return Err(beat);
            }
        }

        pub fn pop(&mut self) -> Result<u32, ()> {
            if self.curr_len <= 0 {
                return Err(());
            } else if let Some(Some(beat)) = self.q.pop_back() {
                self.curr_len -= 1;
                return Ok(beat);
            } else {
                return Err(());
            }
        }

        pub fn new(size: u32) -> Self {
            BoundQueue {
                size,
                curr_len: 0,
                q: VecDeque::new(),
            }
        }
    }
}
