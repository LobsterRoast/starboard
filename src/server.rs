use sdl3::gamepad::{Axis, Button};

#[derive(Default)]
pub struct StarboardServerBuilder {
    // A struct to help build a server
    ip: String,
    port: u16,
    enabled_buttons: u32,
    enabled_axes: u32,
}

impl StarboardServerBuilder {
    // Set the target IP of the server
    fn set_ip(self, ip: String) -> Self {
        let mut builder = self;
        builder.ip = ip;
        builder
    }

    // Set the target port of the server
    fn set_port(self, port: u16) -> Self {
        let mut builder = self;
        builder.port = port;
        builder
    }

    // Enable `button` on the server
    fn enable_button(self, button: Button) -> Self {
        let mut builder = self;
        let button_code = button.to_ll().0;
        builder.enabled_buttons |= 1 << button_code;
        builder
    }

    // Enable each button in `buttons` on the server
    fn enable_buttons<T>(self, buttons: T) -> Self
    where
        T: IntoIterator<Item = Button>,
    {
        let mut builder = self;
        for button in buttons {
            builder = builder.enable_button(button);
        }
        builder
    }

    // Enable `axis` on the server
    fn enable_axis(self, axis: Axis) -> Self {
        let mut builder = self;
        let axis_code = axis.to_ll().0;
        builder.enabled_axes |= 1 << axis_code;
        builder
    }

    // Enable each axes in `axes` on the server
    fn enable_axes<T>(self, axes: T) -> Self
    where
        T: IntoIterator<Item = Axis>,
    {
        let mut builder = self;
        for axis in axes {
            builder = builder.enable_axis(axis);
        }
        builder
    }
}

pub struct StarboardServer {
    // The server is what will receive input packets from the controller and simulate a virtual
    // joystick on another PC
    address: String,     // i.e. {ip}:{port}
    enabled_inputs: u32, // Bitmask representing all the enabled buttons on the server
    enabled_axes: u32,   // Bitmask representing all the enabled axes on the
                         // server
}
