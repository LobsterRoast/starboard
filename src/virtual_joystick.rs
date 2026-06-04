use core::convert::TryInto;

use anyhow::Result;
use evdev::{
    AttributeSet, InputEvent, KeyCode,
    uinput::{VirtualDevice, VirtualDeviceBuilder},
};

use crate::{
    bitmask::Bitmask,
    input::{FromByte, StarboardInput},
    virtual_joystick,
};

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
