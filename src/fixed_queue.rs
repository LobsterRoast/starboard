use core::iter::IntoIterator;
use std::{
    collections::{self, VecDeque},
    ops::{Index, IndexMut},
};

// This is a queue with a maximum size, such that - if a push should cause it to exceed the maximum
// size - an element will be popped from the front of the queue until the capacity is met. This is
// used to keep track of things like latency data.
#[derive(Debug, Copy, Clone)]
pub struct FixedQueue<T, const N: usize>
where
    T: Default + Copy + Clone,
{
    inner: [T; N],
    count: usize,
    capacity: usize,
}

impl<T, const N: usize> FixedQueue<T, N>
where
    T: Default + Copy + Clone,
{
    pub fn new() -> Self {
        Self {
            inner: [T::default(); N],
            count: 0,
            capacity: N,
        }
    }

    // Push an element to the back of the queue, and pop from the front if the maximum capacity is
    // exceeded
    pub fn push_back(&mut self, value: T) {
        if self.count < self.capacity {
            self.inner[self.count] = value;
            self.count += 1;
        } else {
            self.shift();
            self.inner[self.capacity - 1] = value;
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // Shift each element over 1 to the right
    // i.e. [1, 2, 3, 4, 5] becomes [2, 3, 4, 5, 5]
    // Note: The last element remains unchanged
    fn shift(&mut self) {
        for i in 0..(self.capacity - 2) {
            self.inner[i] = self.inner[i + 1];
        }
    }
}

impl<T, const N: usize> IntoIterator for FixedQueue<T, N>
where
    T: Default + Copy + Clone,
{
    type Item = T;
    type IntoIter = <[T; N] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<T, const N: usize> Index<usize> for FixedQueue<T, N>
where
    T: Default + Copy + Clone,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for FixedQueue<T, N>
where
    T: Default + Copy + Clone,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}
