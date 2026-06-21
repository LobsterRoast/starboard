use core::iter::IntoIterator;

use crate::fixed_queue::FixedQueue;

#[test]
fn test_full_fixed_queue_into_iter() {
    const N: usize = 5;
    let raw_arr: [u8; N] = [1, 4, 2, 6, 8];
    let mut queue: FixedQueue<u8, N> = FixedQueue::new();
    for i in raw_arr {
        queue.push_back(Some(i));
    }
    let queue_arr: [u8; N] = queue
        .into_iter()
        .map(|option| option.unwrap())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();
    assert_eq!(queue_arr, raw_arr);
}

#[test]
fn test_partial_fixed_queue_into_iter() {
    const N: usize = 5;
    let raw_arr: [u8; 3] = [1, 4, 2];
    let mut queue: FixedQueue<u8, N> = FixedQueue::new();
    for i in raw_arr {
        queue.push_back(Some(i));
    }
    let queue_arr: [u8; 3] = queue
        .into_iter()
        .map(|option| option.unwrap())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();
    assert_eq!(queue_arr, raw_arr);
}

#[test]
fn test_fixed_queue_index() {
    let mut queue: FixedQueue<u8, 5> = FixedQueue::new();
    for i in 0..10 {
        queue.push_back(Some(i));
        println!("{:?}", queue);
    }
    assert_eq!(5, queue[0].unwrap());
}
