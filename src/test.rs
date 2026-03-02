#[cfg(test)]
mod test {
    use crate::packets::*;
    use crate::util::get_ip;

    #[test]
    fn test_get_ip() {
        assert!(get_ip("0.0.0.0".to_string(), &"1.1.1.1".to_string()) == "1.1.1.1");
        assert!(get_ip("0.0.0.0".to_string(), &"".to_string()) == "0.0.0.0");
    }

    #[test]
    fn test_serialization() {
        let packet = InputPacket::default();

        let serialized_packet =
            serialize::<InputPacket>(packet).expect("Failed to serialize packet");

        let deserialized_packet =
            deserialize::<InputPacket>(serialized_packet).expect("Failed to deserialize packet");

        assert_eq!(InputPacket::default(), deserialized_packet);
    }
}
