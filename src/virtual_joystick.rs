use sdl3::{
    JoystickSubsystem, Sdl,
    joystick::{Joystick, VirtualJoystickConnection},
};

// Wrapper for virtual joystick interactions
pub struct VirtualJoystick {
    sdl_context: Sdl,
    joystick_subsystem: JoystickSubsystem,
    virtual_joystick_connection: VirtualJoystickConnection,
    virtual_joystick: Joystick,
}
