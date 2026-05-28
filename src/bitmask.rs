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
}
