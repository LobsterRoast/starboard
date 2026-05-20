use anyhow::Result;
use sdl3::{
    JoystickSubsystem, Sdl,
    gamepad::{Axis, Button},
    joystick::{Joystick, VirtualJoystickConnection, VirtualJoystickDescription},
};

use crate::virtual_joystick;

// Wrapper for virtual joystick interactions
pub struct VirtualJoystick {
    sdl_context: Sdl,
    joystick_subsystem: JoystickSubsystem,
    virtual_joystick_connection: VirtualJoystickConnection,
    virtual_joystick: Joystick,
}

impl VirtualJoystick {}

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
