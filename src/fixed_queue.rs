use std::collections::VecDeque;

// This is a queue with a maximum size, such that - if a push should cause it to exceed the maximum
// size - an element will be popped from the front of the queue until the capacity is met. This is
// used to keep track of things like latency data.
pub struct FixedQueue<T> {
    inner: VecDeque<T>,
    capacity: usize,
}
