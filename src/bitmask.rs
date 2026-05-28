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
    raw: BitmaskRaw,
    size: usize,
}

impl Bitmask {
    pub fn new(size: usize) -> Self {
        let mut raw = BitmaskRaw::EIGHT { raw: 0 };
        if size > 64 {
            raw = BitmaskRaw::ONETWENTYEIGHT { raw: 0 };
        } else if size > 32 {
            raw = BitmaskRaw::SIXTYFOUR { raw: 0 };
        } else if size > 16 {
            raw = BitmaskRaw::THIRTYTWO { raw: 0 };
        } else if size > 8 {
            raw = BitmaskRaw::SIXTEEN { raw: 0 };
        }
        Self { raw, size }
    }

    // Public interface for the read_bit() function of BitmaskRaw
    #[inline]
    pub fn read_bit(&self, index: usize) -> bool {
        if &index >= &self.size {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                self.size, index
            );
        }
        self.raw.read_bit(index)
    }
}
