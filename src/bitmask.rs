use core::usize;

enum BitmaskRaw {
    EIGHT { raw: u8 },
    SIXTEEN { raw: u16 },
    THIRTYTWO { raw: u32 },
    SIXTYFOUR { raw: u64 },
    ONETWENTYEIGHT { raw: u128 },
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
}
