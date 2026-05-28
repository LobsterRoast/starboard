use anyhow::Result;
use evdev::{
    AttributeSet, KeyCode,
    uinput::{VirtualDevice, VirtualDeviceBuilder},
};
use sdl3::{
    JoystickSubsystem, Sdl,
    gamepad::{Axis, Button},
    joystick::{Joystick, VirtualJoystickConnection, VirtualJoystickDescription},
};

use crate::{
    input::{FromByte, StarboardInput},
    virtual_joystick,
};

// Wrapper for virtual joystick interactions
pub struct VirtualJoystick {
    sdl_context: Sdl,
    joystick_subsystem: JoystickSubsystem,
    virtual_joystick_connection: VirtualJoystickConnection,
    virtual_joystick: Joystick,
}

impl VirtualJoystick {
    pub fn new<B, A>(buttons: B, axes: A) -> Result<Self>
    where
        B: IntoIterator<Item = Button>,
        A: IntoIterator<Item = Axis>,
    {
        let desc = gen_description(buttons, axes);
        let sdl_context = sdl3::init()?;
        let joystick_subsystem = sdl_context.joystick()?;
        let virtual_joystick_connection = joystick_subsystem.attach_virtual_joystick(desc)?;
        let id = virtual_joystick_connection.id();
        let virtual_joystick = joystick_subsystem.open(id)?;

        Ok(Self {
            sdl_context,
            joystick_subsystem,
            virtual_joystick_connection,
            virtual_joystick,
        })
    }

    // Sends `input` to your system's input handling
    pub fn send_input(&self, input: StarboardInput) -> Result<()> {
        let virtual_joystick = &self.virtual_joystick;
        match input {
            StarboardInput::Button { id, value } => {
                virtual_joystick.set_virtual_button(id, value)?
            }
            StarboardInput::Axis { id, value } => virtual_joystick.set_virtual_axis(id, value)?,
        };
        Ok(())
    }
}

// Generates the description that will be used to build a virtual joystick
fn gen_description<B, A>(buttons: B, axes: A) -> VirtualJoystickDescription
where
    B: IntoIterator<Item = Button>,
    A: IntoIterator<Item = Axis>,
{
    let desc = VirtualJoystickDescription::new()
        .name("Starboard Virtual Gamepad")
        .with_buttons(buttons)
        .with_axes(axes);
    desc
}

// Wrapper for Virtual Joysticks using uinput instead of SDL3
pub struct VirtualJoystickEvdev {
    raw: VirtualDevice,
}

// Builder struct for VirtualJoystickEvdev
pub struct VirtualJoystickEvdevBuilder<'a> {
    raw: VirtualDeviceBuilder<'a>,
}

impl VirtualJoystickEvdevBuilder<'_> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            raw: VirtualDevice::builder()?,
        })
    }

    // Build a VirtualJoystickEvdev
    pub fn build(self) -> Result<VirtualJoystickEvdev> {
        Ok(VirtualJoystickEvdev {
            raw: { self.raw.build()? },
        })
    }

    // Enable all valid buttons in `buttons`
    // TODO: Make buttons a u16 for optimization purposes
    pub fn enable_buttons_bitmask(self, buttons: u32) -> Result<Self> {
        let mut attribute_set: AttributeSet<KeyCode> = AttributeSet::new();
        for i in 0..32 {
            let pow = 1 << i;
            if buttons & pow <= 0 {
                continue;
            }
            if let Ok(key_code) = FromByte::<KeyCode>::from_byte(pow) {
                attribute_set.insert(key_code);
            }
        }
        let raw = self.raw.with_keys(&attribute_set)?;
        Ok(Self { raw })
    }

    // Enable all valid axes in `axes`
    pub fn enable_axes_bitmask(self, axes: u32) -> Result<Self> {
        let mut raw = self.raw;
        for i in 0..32 {
            let pow = 1 << i;
            if axes & pow <= 0 {
                continue;
            }
            raw = raw.with_absolute_axis(&pow.from_byte()?)?;
        }
        Ok(Self { raw })
    }
}
