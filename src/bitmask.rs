use core::{ops::Index, slice::SliceIndex, usize};

enum BitmaskRaw {
    EIGHT { raw: u8 },
    SIXTEEN { raw: u16 },
    THIRTYTWO { raw: u32 },
    SIXTYFOUR { raw: u64 },
    ONETWENTYEIGHT { raw: u128 },
}

impl BitmaskRaw {
    // Returns the value of the bit an index `index`
    // The index trait doesn't work here since the
    // returned value must be owned
    #[inline]
    fn read_bit(&self, index: usize) -> bool {
        match self {
            BitmaskRaw::EIGHT { raw } => raw & (1 << index) != 0,
            BitmaskRaw::SIXTEEN { raw } => raw & (1 << index) != 0,
            BitmaskRaw::THIRTYTWO { raw } => raw & (1 << index) != 0,
            BitmaskRaw::SIXTYFOUR { raw } => raw & (1 << index) != 0,
            BitmaskRaw::ONETWENTYEIGHT { raw } => raw & (1 << index) != 0,
        }
    }
}

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
        if &index >= &self.size {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                self.size, index
            );
        }
        self.raw & (1 << index) != 0
    }
}
