use core::{fmt::Debug, hash::Hash};
use evdev::{AbsoluteAxisCode, KeyCode};
use heapless::index_map::FnvIndexMap;
use std::sync::LazyLock;

const SUPPORTED_BUTTONS_BLUEPRINT: [KeyCode; 17] = [
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
    KeyCode::BTN_TRIGGER_HAPPY4,
    KeyCode::BTN_MODE,
    KeyCode::BTN_THUMB,
    KeyCode::BTN_THUMB2,
];

pub const BUTTON_COUNT: u32 = SUPPORTED_BUTTONS_BLUEPRINT.len() as u32;

const SUPPORTED_AXES_BLUEPRINT: [AbsoluteAxisCode; 8] = [
    AbsoluteAxisCode::ABS_X,
    AbsoluteAxisCode::ABS_Y,
    AbsoluteAxisCode::ABS_Z,
    AbsoluteAxisCode::ABS_RX,
    AbsoluteAxisCode::ABS_RY,
    AbsoluteAxisCode::ABS_RZ,
    AbsoluteAxisCode::ABS_HAT0X,
    AbsoluteAxisCode::ABS_HAT0Y,
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
