use crate::{
    bitmask::Bitmask,
    datagram::{deserialize, serialize},
    input::{StarboardAxisStates, StarboardButtonStates, StarboardInputPacket},
};

fn test_button_states() -> StarboardButtonStates {
    let mut raw = Bitmask::new(4);
    raw.write_bit(0, true);
    raw.write_bit(2, true);
    raw.write_bit(3, true);
    StarboardButtonStates { raw }
}

const TEST_AXIS_STATES: StarboardAxisStates = StarboardAxisStates {
    axes: [0, 101, -63, 112, -1, 127, 0, 0],
};

#[test]
fn test_packet_serialization_symmetry() {
    let packet = StarboardInputPacket {
        buttons: test_button_states(),
        axes: TEST_AXIS_STATES,
        id: 0,
    };

    let raw = serialize(&packet).unwrap();
    assert_eq!(packet, deserialize(raw).unwrap());
}
