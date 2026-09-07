pub mod queue {
    use std::collections::VecDeque;
    struct BoundQueue {
        size: u32,
        curr_len: u32,
        q: VecDeque<Option<u32>>,
    }
   
    impl BoundQueue {
        fn push(&mut self, beat: u32) -> Result<(), u32> {
            if self.curr_len < self.size {
                self.q.push_front(Some(beat));
                self.curr_len += 1;
                return Ok(());
            } else {
                return Err(beat);
            }
        }

        fn pop(&mut self) -> Result<u32, ()> {
            if self.curr_len <= 0 {
                return Err(());
            } else if let Some(Some(beat)) = self.q.pop_back() {
                self.curr_len -= 1;
                return Ok(beat);
            } else {
                return Err(());
            }
        }

        fn new(size: u32) -> Self {
            BoundQueue {
                size,
                curr_len: 0,
                q: VecDeque::new(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn push_then_pop() {
            let mut q = BoundQueue::new(4);
            assert!(q.push(42).is_ok());
            assert_eq!(q.pop(), Ok(42));
        }

        #[test]
        fn push_when_full() {
            let mut q = BoundQueue::new(1);
            assert!(q.push(1).is_ok());
            assert_eq!(q.push(2), Err(2));
        }

        #[test]
        fn pop_when_empty() {
            let mut q = BoundQueue::new(4);
            assert_eq!(q.pop(), Err(()));
        }

        #[test]
        fn push_three_pop_three() {
            let mut q = BoundQueue::new(4);
            assert!(q.push(1).is_ok());
            assert!(q.push(2).is_ok());
            assert!(q.push(3).is_ok());
            assert_eq!(q.pop(), Ok(1));
            assert_eq!(q.pop(), Ok(2));
            assert_eq!(q.pop(), Ok(3));
        }
    }
}
