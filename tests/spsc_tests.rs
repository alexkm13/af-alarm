use af_alarm::spsc::queue::BoundQueue;

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
