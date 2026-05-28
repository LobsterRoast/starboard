// This struct will contain the public
// interface for using a bitmask
pub struct Bitmask {
    raw: u32,
    size: usize,
}

impl Bitmask {
    pub fn new(size: usize) -> Self {
        if size > 32 {
            panic!("Bitask size cannot exceed 32");
        }
        Self { raw: 0, size }
    }

    // Check that the index-th bit of raw is set to 1
    #[inline]
    pub fn read_bit(&self, index: usize) -> bool {
        self.check_index(index);
        self.raw & (1 << index) != 0
    }

    // Checks if the index is valid and panics if not
    #[inline]
    fn check_index(&self, index: usize) {
        if &index >= &self.size {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                self.size, index
            );
        }
    }
}
