use bincode::{Decode, Encode};

// This struct will contain the public
// interface for using a bitmask
#[derive(Debug, Decode, Encode, Copy, Clone)]
pub struct Bitmask {
    raw: u32,
    size: u32,
}

impl Bitmask {
    pub const MAX: Self = Self {
        raw: u32::MAX,
        size: 32,
    };

    pub fn new(size: u32) -> Self {
        if size > 32 {
            panic!("Bitask size cannot exceed 32");
        }
        Self { raw: 0, size }
    }

    // Generate from a predefined u32
    pub fn new_from_u32(size: u32, raw: u32) -> Self {
        Self { raw, size }
    }

    // Build a bitmask from a raw u32
    pub fn from_ll(raw: u32, size: u32) -> Self {
        if size > 32 {
            panic!("Bitask size cannot exceed 32");
        }
        Self { raw, size }
    }

    // Check that the index-th bit of raw is set to 1
    #[inline]
    pub fn read_bit(&self, index: u32) -> bool {
        self.check_index(index);
        self.raw & (1 << index) != 0
    }

    // Set the index-th bit of raw to value
    #[inline]
    pub fn write_bit(&mut self, index: u32, value: bool) {
        self.check_index(index);
        let current_state = self.read_bit(index);
        if current_state == value {
            return;
        } else if current_state {
            self.raw ^= 1 << index;
        } else {
            self.raw |= 1 << index;
        }
    }

    // Checks if the index is valid and panics if not
    #[inline]
    fn check_index(&self, index: u32) {
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
    bit: u32,
    size: u32,
}

impl Iterator for BitmaskIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.bit += 1;
        if self.bit - 1 >= self.size {
            None
        } else {
            Some((self.mask & (1 << (self.bit - 1))) != 0)
        }
    }
}
