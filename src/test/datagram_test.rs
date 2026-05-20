use crate::{
    datagram::{deserialize, serialize},
    input::{StarboardAxisStates, StarboardButtonStates, StarboardInput, StarboardInputPacket},
};

const TEST_BUTTON_STATES: StarboardButtonStates = StarboardButtonStates { raw: 13 }; // 0b1101
const TEST_AXIS_STATES: StarboardAxisStates = StarboardAxisStates {
    axes: [0, 145, 223, 1125, 102, 255],
};

#[test]
fn test_button_state_deserialization() {
    let raw: Vec<u8> = vec![0b1101];
    assert_eq!(TEST_BUTTON_STATES, deserialize(raw).unwrap());
}

#[test]
fn test_button_state_serialization() {
    let raw: Vec<u8> = vec![0b1101];
    assert_eq!(raw, serialize(TEST_BUTTON_STATES).unwrap());
}

#[test]
fn test_packet_serialization_symmetry() {
    let packet = StarboardInputPacket {
        buttons: TEST_BUTTON_STATES,
        axes: TEST_AXIS_STATES,
    };

    let raw = serialize(&packet).unwrap();
    assert_eq!(packet, deserialize(raw).unwrap());
}
