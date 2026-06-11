use core::{convert::TryInto, iter::IntoIterator};

use anyhow::Result;
use evdev::{
    AbsoluteAxisCode, AttributeSet, Device, InputEvent, KeyCode, enumerate,
    uinput::{VirtualDevice, VirtualDeviceBuilder},
};

use crate::{
    bitmask::Bitmask,
    input::{FromByte, StarboardInput},
};

pub const SUPPORTED_BUTTONS: [KeyCode; 14] = [
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
];

pub const SUPPORTED_AXES: [AbsoluteAxisCode; 8] = [
    AbsoluteAxisCode::ABS_X,
    AbsoluteAxisCode::ABS_Y,
    AbsoluteAxisCode::ABS_Z,
    AbsoluteAxisCode::ABS_RX,
    AbsoluteAxisCode::ABS_RY,
    AbsoluteAxisCode::ABS_RZ,
    AbsoluteAxisCode::ABS_HAT0X,
    AbsoluteAxisCode::ABS_HAT0Y,
];

// Wrapper for Virtual Joysticks using uinput instead of SDL3
pub struct VirtualJoystick {
    raw: VirtualDevice,
}

impl VirtualJoystick {
    // Sends `input` to your system's input handling
    pub fn send_input(&mut self, input: StarboardInput) -> Result<()> {
        let event: InputEvent = input.try_into()?;
        self.raw.emit(&[event])?;
        Ok(())
    }
}

// Builder struct for VirtualJoystick
pub struct VirtualJoystickBuilder<'a> {
    raw: VirtualDeviceBuilder<'a>,
}

impl VirtualJoystickBuilder<'_> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            raw: VirtualDevice::builder()?,
        })
    }

    // Build a VirtualJoystick
    pub fn build(self) -> Result<VirtualJoystick> {
        Ok(VirtualJoystick {
            raw: { self.raw.build()? },
        })
    }

    // Enable all valid buttons in `buttons`
    pub fn enable_buttons_bitmask(self, buttons: Bitmask) -> Result<Self> {
        let mut attribute_set: AttributeSet<KeyCode> = AttributeSet::new();
        for (bit, state) in buttons.into_iter().enumerate() {
            if state {
                continue;
            }
            if let Ok(key_code) = FromByte::<KeyCode>::from_byte(1 << bit) {
                attribute_set.insert(key_code);
            }
        }
        let raw = self.raw.with_keys(&attribute_set)?;
        Ok(Self { raw })
    }

    // Enable all valid axes in `axes`
    pub fn enable_axes_bitmask(self, axes: Bitmask) -> Result<Self> {
        let mut raw = self.raw;
        for (bit, state) in axes.into_iter().enumerate() {
            if state {
                continue;
            }
            raw = raw.with_absolute_axis(&(1 << bit).from_byte()?)?;
        }
        Ok(Self { raw })
    }
}

// Takes a device and gives it a score based on how closely it resembles a Steam Deck's layout
fn get_device_supported_attributes_score(device: &Device) -> u8 {
    let supported_buttons = device.supported_keys().unwrap();
    let supported_axes = device.supported_absolute_axes().unwrap();

    let mut score: u8 = 0;
    SUPPORTED_BUTTONS
        .into_iter()
        .filter(|code| supported_buttons.contains(*code))
        .for_each(|_| score += 1);
    SUPPORTED_AXES
        .into_iter()
        .filter(|code| supported_axes.contains(*code))
        .for_each(|_| score += 1);
    score
}

// Iterates through evdev devices and picks the one that looks most like a Steam Deck
pub fn find_best_evdev_device() -> Result<Device> {
    Ok(enumerate()
        .map(|(_, device)| device)
        .max_by_key(|device| get_device_supported_attributes_score(&device))
        .unwrap())
}

// Wrapper for evdev::Device
pub struct DeviceWrapper {
    device: Device,
    supported_buttons: Vec<KeyCode>,
    supported_axes: Vec<AbsoluteAxisCode>,
}

impl DeviceWrapper {
    // Returns the state of each supported button on the device
    pub fn get_button_states(&self) -> Result<Vec<(KeyCode, bool)>> {
        let attr_set = self.device.get_key_state()?;
        let mut button_states: Vec<(KeyCode, bool)> = Vec::new();
        self.supported_buttons
            .iter()
            .map(|button| *button)
            .for_each(|button| button_states.push((button, attr_set.contains(button))));
        Ok(button_states)
    }

    // Returns a vector of StarboardInputs representing the state of every supported button
    pub fn get_button_inputs(&self) -> Result<Vec<StarboardInput>> {
        let attr_set = self.device.get_key_state()?;
        let mut inputs: Vec<StarboardInput> = Vec::new();
        self.supported_buttons
            .iter()
            .map(|button| *button)
            .for_each(|button| {
                inputs.push(StarboardInput::Button {
                    id: button.0.into(),
                    value: attr_set.contains(button),
                })
            });
        Ok(inputs)
    }

    // Returns the state of each supported axis on the device
    pub fn get_axis_states(&self) -> Result<Vec<(AbsoluteAxisCode, i32)>> {
        let states = self.device.get_abs_state()?;
        let mut axis_states: Vec<(AbsoluteAxisCode, i32)> = Vec::new();
        self.supported_axes
            .iter()
            .map(|axis| *axis)
            .for_each(|axis| axis_states.push((axis, states[axis.0 as usize].value)));
        Ok(axis_states)
    }
}
