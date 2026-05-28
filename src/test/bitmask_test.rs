use crate::bitmask::Bitmask;

#[test]
fn test_bitmask_read_write() {
    let mut bitmask = Bitmask::new(5);
    assert!(!bitmask.read_bit(0));
    bitmask.write_bit(0, true);
    assert!(bitmask.read_bit(0));
}
