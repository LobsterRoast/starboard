use sdl2::controller::{Button, Axis};

use evdev::{KeyCode, AbsoluteAxisCode};

use std::sync::{Arc, OnceLock};
use std::collections::HashMap;

// This macro is essentially just a version of println! that will only run if DEBUG_MODE is True.
#[macro_export] macro_rules! debug {
    ($fmt:expr, $($args:tt)*) => {
        if *DEBUG_MODE.get().unwrap() {
            println!(concat!("[DEBUG] ", $fmt), $($args)*);
        }
    };
    ($fmt:expr) => {
        if *DEBUG_MODE.get().unwrap() {
            println!(concat!("[DEBUG] ", $fmt));   
        }
    };
}

pub static DEBUG_MODE: OnceLock<bool> = OnceLock::new();

pub static SDL_KEY_ASSOCIATIONS: OnceLock<HashMap<Button, u16>> = OnceLock::new();

// The client side cannot use evdev to get controller inputs
// This is because evdev cannot read the triggers on the back of the Steam Deck
// Also, for whatever reason, the evdev device for the Steam Deck only seems to work
// with some sort of Steam game open.
pub const SDL_KEYS: [Button; 14] = [
    Button::Y,
    Button::A,
    Button::B,
    Button::X,
    Button::LeftStick,
    Button::RightStick,
    Button::LeftShoulder,
    Button::RightShoulder,
    Button::Start,
    Button::Guide,
    Button::Paddle1,
    Button::Paddle2,
    Button::Paddle3,
    Button::Paddle4
];

pub const EVDEV_KEYS: [KeyCode; 14] = [
    KeyCode::BTN_NORTH,
    KeyCode::BTN_SOUTH,
    KeyCode::BTN_EAST,
    KeyCode::BTN_WEST,
    KeyCode::BTN_THUMBL,
    KeyCode::BTN_THUMBR,
    KeyCode::BTN_TL,
    KeyCode::BTN_TR,
    KeyCode::BTN_START,
    KeyCode::BTN_SELECT,
    KeyCode::BTN_TRIGGER_HAPPY1,
    KeyCode::BTN_TRIGGER_HAPPY2,
    KeyCode::BTN_TRIGGER_HAPPY3,
    KeyCode::BTN_TRIGGER_HAPPY4
];

pub const BIN_KEYS: [u16; 14] = [
    0b0000000000000001,
    0b0000000000000010,
    0b0000000000000100,
    0b0000000000001000,
    0b0000000000010000,
    0b0000000000100000,
    0b0000000001000000,
    0b0000000010000000,
    0b0000000100000000,
    0b0000001000000000,
    0b0000010000000000,
    0b0000100000000000,
    0b0001000000000000,
    0b0010000000000000
];

// Iterable constant array of all the analog values that will be used. Absolute is code for Analog in this case.
// For some reason, the D-Pad is an analog value. ABS_HAT0(X/Y) refers to the D-Pad values.
pub const SDL_AXES: [Axis; 6] = [
    Axis::LeftX,
    Axis::LeftY,
    Axis::TriggerLeft,
    Axis::RightX,
    Axis::RightY,
    Axis::TriggerRight
];

pub const EVDEV_AXES: [AbsoluteAxisCode; 8] = [
    AbsoluteAxisCode::ABS_X,
    AbsoluteAxisCode::ABS_Y,
    AbsoluteAxisCode::ABS_Z,
    AbsoluteAxisCode::ABS_RX,
    AbsoluteAxisCode::ABS_RY,
    AbsoluteAxisCode::ABS_RZ,
    AbsoluteAxisCode::ABS_HAT0X,
    AbsoluteAxisCode::ABS_HAT0Y
];

#[derive(bincode::Encode, bincode::Decode)]
pub struct Packet {
    pub key_states: u16,
    pub abs_states: [i32; 8],
    pub timestamp: String
}

#[derive(Default)]
pub struct States {
    pub key_states: u16,
    pub abs_states: HashMap<AbsoluteAxisCode, i32>
}

impl Packet {
    pub fn new(key_states: u16, abs_states: [i32; 8], timestamp: String) -> Self {
        Self {
                key_states: key_states,
                abs_states: abs_states,
                timestamp: timestamp
        }
    }
}

impl States {
    pub fn new() -> States {
        let mut states: States = Default::default();
        states.key_states = 0;
        states.abs_states = HashMap::new();
        for abs in EVDEV_AXES {
            states.abs_states.insert(abs, 0);
        }
        states
    }
}

pub fn get_ip(default: String, ip: Arc<String>) -> String {
    if *ip == "".to_string(){
        return default;
    }
    else {
        return ip.to_string();
    }
}

pub fn get_key_associations() -> &'static HashMap<Button, u16> {
    SDL_KEY_ASSOCIATIONS.get_or_init(|| {
        let mut map: HashMap<Button, u16> = HashMap::new();
        for i in 0..BIN_KEYS.len() {
            map.insert(SDL_KEYS[i], BIN_KEYS[i]);
        }
        map
    })
}