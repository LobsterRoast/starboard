use anyhow::Result;
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use sdl3::{
    JoystickSubsystem, Sdl,
    gamepad::{Axis, Button},
    joystick::{Joystick, VirtualJoystickConnection, VirtualJoystickDescription},
};

use crate::{input::StarboardInput, virtual_joystick};

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
}
