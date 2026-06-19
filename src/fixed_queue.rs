use std::collections::VecDeque;

// This is a queue with a maximum size, such that - if a push should cause it to exceed the maximum
// size - an element will be popped from the front of the queue until the capacity is met. This is
// used to keep track of things like latency data.
pub struct FixedQueue<T> {
    inner: VecDeque<T>,
    capacity: usize,
}

impl<T> FixedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: VecDeque::new(),
            capacity,
        }
    }

    // Push an element to the back of the queue, and pop from the front if the maximum capacity is
    // exceeded
    pub fn push_back(&mut self, value: T) {
        self.inner.push_back(value);
        if self.inner.len() > self.capacity {
            self.inner.pop_front();
        }
    }
}
