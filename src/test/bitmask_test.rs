use crate::bitmask::Bitmask;

#[test]
fn test_bitmask_read_write() {
    let mut bitmask = Bitmask::new(5);
    assert!(!bitmask.read_bit(0));
    bitmask.write_bit(0, true);
    assert!(bitmask.read_bit(0));
}

#[test]
fn test_bitmask_iteration() {
    let mut bitmask = Bitmask::new(5);
    bitmask.write_bit(0, true);
    bitmask.write_bit(3, true);

    let mut i = 0;
    for bit in bitmask {
        if i == 0 || i == 3 {
            assert!(bit);
        } else {
            assert!(!bit);
        }
        i += 1;
    }
}
