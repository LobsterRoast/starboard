// This represents a string with a maximum capacity that can be stored without a heap allocation
#[derive(Debug, Copy, Clone)]
pub struct StarboardString {
    inner: [char; 30],
    count: usize,
}
