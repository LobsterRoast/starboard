use sdl3::{
    gamepad::{Axis, Button},
    joystick::Joystick,
};

use crate::input::{StarboardInput, StarboardInputPacket};

pub struct StarboardServerBuilder {
    // A struct to help build a server
    ip: [u8; 4],
    port: u16,
    enabled_buttons: u32,
    enabled_axes: u32,
}

impl StarboardServerBuilder {
    fn new() -> Self {
        Self {
            ip: [0; 4],
            port: 8080,
            enabled_buttons: 0,
            enabled_axes: 0,
        }
    }

    // Build the server
    fn build(self) -> StarboardServer {
        use crate::datagram::format_addr;

        // Defaults to returning 0.0.0.0:0
        let address = format_addr(self.ip, self.port);
        let enabled_buttons = self.enabled_buttons;
        let enabled_axes = self.enabled_axes;

        StarboardServer {
            address,
            enabled_buttons,
            enabled_axes,
        }
    }
    // Set the target IP of the server
    fn set_ip(self, ip: [u8; 4]) -> Self {
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
    address: String,      // i.e. {ip}:{port}
    enabled_buttons: u32, // Bitmask representing all the enabled buttons on the server
    enabled_axes: u32,    // Bitmask representing all the enabled axes on the
                          // server
}

impl StarboardServer {
    // Unpacks a StarboardInputPacket and sends the inputs to your device's input handling
    fn handle_packet(&self, virt_joystick: &Joystick, packet: StarboardInputPacket) {
        let inputs = packet.unpack(self.enabled_buttons, self.enabled_axes);
        for input in inputs {
            self.send_input(virt_joystick, input);
        }
    }

    // Takes a StarboardInput and sends it to a virtual joystick
    // This function acts as the bridge between Starboard and your device's input handling
    fn send_input(&self, virt_joystick: &Joystick, input: StarboardInput) {
        match input {
            StarboardInput::Button { id, value } => virt_joystick.set_virtual_button(id, value),
            StarboardInput::Axis { id, value } => virt_joystick.set_virtual_axis(id, value),
        };
    }
}
