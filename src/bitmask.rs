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

    // Set the index-th bit of raw to value
    #[inline]
    pub fn write_bit(&mut self, index: usize, value: bool) {
        self.check_index(index);
        let current_state = self.read_bit(index);
        if current_state == value {
            return;
        } else if current_state {
            self.raw ^= 1 << index;
        } else {
            self.raw &= 1 << index;
        }
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

impl IntoIterator for Bitmask {
    type Item = bool;
    type IntoIter = BitmaskIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitmaskIterator {
            mask: self.raw,
            bit: 0,
            size: self.size,
        }
    }
}

// Iterator struct for bitmask so
// that you can iterate through a
// bitmask's data
pub struct BitmaskIterator {
    mask: u32,
    bit: usize,
    size: usize,
}

impl Iterator for BitmaskIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.bit += 1;
        if self.bit >= self.size {
            None
        } else {
            Some(self.mask & (1 << self.bit) != 0)
        }
    }
}
