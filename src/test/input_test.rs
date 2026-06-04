use core::{assert_eq, convert::TryInto};

use crate::{
    bitmask::Bitmask,
    input::{
        FromByte, IntoByte, StarboardAxisStates, StarboardButtonStates, StarboardInput,
        StarboardInputPacket,
    },
};
use evdev::{AbsoluteAxisCode, EventType, InputEvent, KeyCode};

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
    let mask = Bitmask::new_from_u32(4, 0b1010);
    let states = TEST_BUTTON_STATES.get_state_with_mask(mask);
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
    let mask = Bitmask::new_from_u32(6, 0b100101);
    let states = TEST_AXIS_STATES.get_state_with_mask(mask);
    assert_eq!(states[0], StarboardInput::Axis { id: 0, value: 0 });
    assert_eq!(states[1], StarboardInput::Axis { id: 2, value: 223 });
    assert_eq!(states[2], StarboardInput::Axis { id: 5, value: 255 });
}

#[test]
fn test_packet_unwrap() {
    let packet = StarboardInputPacket {
        buttons: TEST_BUTTON_STATES,
        axes: TEST_AXIS_STATES,
        id: 0,
    };

    let buttons = TEST_BUTTON_STATES.get_state_with_mask(Bitmask::MAX);
    let axes = TEST_AXIS_STATES.get_state_with_mask(Bitmask::MAX);

    let mut combined_inputs: Vec<StarboardInput> = Vec::new();

    for button in buttons {
        combined_inputs.push(button);
    }

    for axis in axes {
        combined_inputs.push(axis);
    }

    assert_eq!(packet.unpack(Bitmask::MAX, Bitmask::MAX), combined_inputs);
}

#[test]
fn test_into_byte_evdev() {
    assert_eq!(KeyCode::BTN_NORTH.into_byte().unwrap(), 1);
    assert_eq!(KeyCode::BTN_TRIGGER_HAPPY1.into_byte().unwrap(), 1024);
}

#[test]
fn test_from_byte_evdev() {
    let key_code: KeyCode = 1024.from_byte().unwrap();
    assert_eq!(key_code, KeyCode::BTN_TRIGGER_HAPPY1);
}

#[test]
#[should_panic]
fn test_into_byte_evdev_panics() {
    KeyCode::KEY_BRIGHTNESS_MAX.into_byte().unwrap();
}

#[test]
#[should_panic]
fn test_from_byte_evdev_panics() {
    FromByte::<KeyCode>::from_byte(246).unwrap();
}

#[test]
fn test_starboard_input_into_evdev_input() {
    let starboard_input = StarboardInput::Axis { id: 0, value: 50 };
    let evdev_input: InputEvent = starboard_input.try_into().unwrap();

    assert_eq!(evdev_input.event_type(), EventType::ABSOLUTE);
    assert_eq!(evdev_input.code(), AbsoluteAxisCode::ABS_X.0);
    assert_eq!(evdev_input.value(), 50);

    let starboard_input = StarboardInput::Button { id: 0, value: true };
    let evdev_input: InputEvent = starboard_input.try_into().unwrap();

    assert_eq!(evdev_input.event_type(), EventType::KEY);
    assert_eq!(evdev_input.code(), KeyCode::BTN_NORTH.0);
    assert_eq!(evdev_input.value(), 1);
}
