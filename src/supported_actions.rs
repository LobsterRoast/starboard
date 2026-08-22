use auto_const_array::auto_const_array_attr as auto_const_array;
use core::{fmt::Debug, hash::Hash};
use evdev::{AbsoluteAxisCode, KeyCode};
use heapless::index_map::FnvIndexMap;
use std::sync::LazyLock;

#[auto_const_array]
const SUPPORTED_BUTTONS_BLUEPRINT: [KeyCode; _] = [
    KeyCode::BTN_THUMB,
    KeyCode::BTN_THUMB2,
    KeyCode::BTN_BASE,
    KeyCode::BTN_SOUTH,
    KeyCode::BTN_EAST,
    KeyCode::BTN_NORTH,
    KeyCode::BTN_WEST,
    KeyCode::BTN_TL,
    KeyCode::BTN_TR,
    KeyCode::BTN_TL2,
    KeyCode::BTN_TR2,
    KeyCode::BTN_SELECT,
    KeyCode::BTN_START,
    KeyCode::BTN_MODE,
    KeyCode::BTN_THUMBL,
    KeyCode::BTN_THUMBR,
    KeyCode::BTN_DPAD_UP,
    KeyCode::BTN_DPAD_DOWN,
    KeyCode::BTN_DPAD_LEFT,
    KeyCode::BTN_DPAD_RIGHT,
    KeyCode::BTN_TRIGGER_HAPPY1,
    KeyCode::BTN_TRIGGER_HAPPY2,
    KeyCode::BTN_TRIGGER_HAPPY3,
    KeyCode::BTN_TRIGGER_HAPPY4,
];

pub const BUTTON_COUNT: u32 = SUPPORTED_BUTTONS_BLUEPRINT.len() as u32;

#[auto_const_array]
const SUPPORTED_AXES_BLUEPRINT: [AbsoluteAxisCode; _] = [
    AbsoluteAxisCode::ABS_X,
    AbsoluteAxisCode::ABS_Y,
    AbsoluteAxisCode::ABS_RX,
    AbsoluteAxisCode::ABS_RY,
    AbsoluteAxisCode::ABS_HAT0X,
    AbsoluteAxisCode::ABS_HAT0Y,
    AbsoluteAxisCode::ABS_HAT1X,
    AbsoluteAxisCode::ABS_HAT1Y,
    AbsoluteAxisCode::ABS_HAT2X,
    AbsoluteAxisCode::ABS_HAT2Y,
];

pub const AXIS_COUNT: u32 = SUPPORTED_AXES_BLUEPRINT.len() as u32;

fn gen_support_map<T, const N: usize>(source: [T; N]) -> FnvIndexMap<T, usize, 32>
where
    T: Eq + Copy + Hash + Debug,
{
    let mut map = FnvIndexMap::new();
    for (i, value) in source.iter().enumerate() {
        map.insert(*value, i).unwrap();
    }
    map
}

pub static SUPPORTED_BUTTONS: LazyLock<FnvIndexMap<KeyCode, usize, 32>> =
    LazyLock::new(|| gen_support_map(SUPPORTED_BUTTONS_BLUEPRINT));

pub static SUPPORTED_AXES: LazyLock<FnvIndexMap<AbsoluteAxisCode, usize, 32>> =
    LazyLock::new(|| gen_support_map(SUPPORTED_AXES_BLUEPRINT));
