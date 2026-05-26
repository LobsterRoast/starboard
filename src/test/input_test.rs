use crate::input::{
    FromByte, IntoByte, StarboardAxisStates, StarboardButtonStates, StarboardInput,
    StarboardInputPacket,
};
use evdev::KeyCode;

const TEST_BUTTON_STATES: StarboardButtonStates = StarboardButtonStates { raw: 13 }; // 0b1101
const TEST_AXIS_STATES: StarboardAxisStates = StarboardAxisStates {
    axes: [0, 145, 223, 1125, 102, 255],
};

#[test]
fn test_button_get_state() {
    assert_eq!(
        TEST_BUTTON_STATES.get_state(0),
        StarboardInput::Button { id: 0, value: true }
    );

    assert_eq!(
        TEST_BUTTON_STATES.get_state(1),
        StarboardInput::Button {
            id: 1,
            value: false
        }
    );

    assert_eq!(
        TEST_BUTTON_STATES.get_state(2),
        StarboardInput::Button { id: 2, value: true }
    );

    assert_eq!(
        TEST_BUTTON_STATES.get_state(3),
        StarboardInput::Button { id: 3, value: true }
    );
}

#[test]
fn test_button_get_states() {
    let states = TEST_BUTTON_STATES.get_state_with_mask(0b1010);
    assert_eq!(
        states,
        vec![
            StarboardInput::Button {
                id: 1,
                value: false
            },
            StarboardInput::Button { id: 3, value: true }
        ]
    )
}

#[test]
fn test_axis_get_state() {
    assert_eq!(
        TEST_AXIS_STATES.get_state(0),
        StarboardInput::Axis { id: 0, value: 0 }
    );

    assert_eq!(
        TEST_AXIS_STATES.get_state(1),
        StarboardInput::Axis { id: 1, value: 145 }
    );

    assert_eq!(
        TEST_AXIS_STATES.get_state(2),
        StarboardInput::Axis { id: 2, value: 223 }
    );

    assert_eq!(
        TEST_AXIS_STATES.get_state(3),
        StarboardInput::Axis { id: 3, value: 1125 }
    );

    assert_eq!(
        TEST_AXIS_STATES.get_state(4),
        StarboardInput::Axis { id: 4, value: 102 }
    );

    assert_eq!(
        TEST_AXIS_STATES.get_state(5),
        StarboardInput::Axis { id: 5, value: 255 }
    );
}

#[test]
fn test_axis_get_states() {
    let states = TEST_AXIS_STATES.get_state_with_mask(0b100101);
    assert_eq!(states[0], StarboardInput::Axis { id: 0, value: 0 });
    assert_eq!(states[1], StarboardInput::Axis { id: 2, value: 223 });
    assert_eq!(states[2], StarboardInput::Axis { id: 5, value: 255 });
}

#[test]
fn test_packet_unwrap() {
    let packet = StarboardInputPacket {
        buttons: TEST_BUTTON_STATES,
        axes: TEST_AXIS_STATES,
    };

    let buttons = TEST_BUTTON_STATES.get_state_with_mask(u32::MAX);
    let axes = TEST_AXIS_STATES.get_state_with_mask(u32::MAX);

    let mut combined_inputs: Vec<StarboardInput> = Vec::new();

    for button in buttons {
        combined_inputs.push(button);
    }

    for axis in axes {
        combined_inputs.push(axis);
    }

    assert_eq!(packet.unpack(u32::MAX, u32::MAX), combined_inputs);
}

#[test]
fn test_into_byte_evdev() {
    assert_eq!(KeyCode::BTN_NORTH.into_byte().unwrap(), 1);
    assert_eq!(KeyCode::BTN_TRIGGER_HAPPY1.into_byte().unwrap(), 1024);
}

#[test]
fn test_from_byte_evdev() {
    assert_eq!(1.from_byte().unwrap(), KeyCode::BTN_NORTH);
    assert_eq!(1024.from_byte().unwrap(), KeyCode::BTN_TRIGGER_HAPPY1);
}

#[test]
#[should_panic]
fn test_into_byte_evdev_panics() {
    KeyCode::KEY_BRIGHTNESS_MAX.into_byte().unwrap();
}
