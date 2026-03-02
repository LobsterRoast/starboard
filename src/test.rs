#[cfg(test)]
mod test {
    use crate::util::get_ip;

    #[test]
    fn test_get_ip() {
        assert!(get_ip("0.0.0.0".to_string(), &"1.1.1.1".to_string()) == "1.1.1.1");
        assert!(get_ip("0.0.0.0".to_string(), &"".to_string()) == "0.0.0.0");
    }
}
