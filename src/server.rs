use std::net::UdpSocket;

use sdl3::{
    gamepad::{Axis, Button},
    joystick::Joystick,
};

use crate::{
    bitmask::Bitmask,
    datagram::{deserialize, serialize},
    input::{StarboardInput, StarboardInputPacket},
    virtual_joystick::{self, VirtualJoystick},
};

use anyhow::Result;

pub struct StarboardServerBuilder {
    // A struct to help build a server
    ip: [u8; 4],
    port: u16,
    enabled_buttons: Bitmask,
    enabled_axes: Bitmask,
}

impl StarboardServerBuilder {
    fn new() -> Self {
        Self {
            ip: [0; 4],
            port: 8080,
            enabled_buttons: Bitmask::new(14),
            enabled_axes: Bitmask::new(14),
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
        let button_code = button.to_ll().0 as u32;
        builder.enabled_buttons.write_bit(button_code, true);
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
        let axis_code = axis.to_ll().0 as u32;
        builder.enabled_axes.write_bit(axis_code, true);
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
    address: String,          // i.e. {ip}:{port}
    enabled_buttons: Bitmask, // Bitmask representing all the enabled buttons on the server
    enabled_axes: Bitmask,    // Bitmask representing all the enabled axes on the
                              // server
}

impl StarboardServer {
    // This is the main loop for the server that receives packets and sends them to the input
    // handling
    async fn server_loop(
        &self,
        buf: &mut [u8; 256],
        virt_joystick: &VirtualJoystick,
        sock: &UdpSocket,
    ) -> Result<()> {
        loop {
            let _ = self.get_packet(buf, sock).await?;
            let raw = Vec::from(&mut *buf);
            let packet: StarboardInputPacket = deserialize(raw)?;
            self.handle_packet(virt_joystick, packet)?;
        }
        Ok(())
    }

    // Waits for a packet to be received and writes the data into `buf`
    async fn get_packet(&self, buf: &mut [u8; 256], sock: &UdpSocket) -> Result<()> {
        while sock.recv(buf)? <= 0 {}
        Ok(())
    }

    // Unpacks a StarboardInputPacket and sends the inputs to your device's input handling
    fn handle_packet(
        &self,
        virt_joystick: &VirtualJoystick,
        packet: StarboardInputPacket,
    ) -> Result<()> {
        let inputs = packet.unpack(self.enabled_buttons, self.enabled_axes);
        for input in inputs {
            virt_joystick.send_input(input)?;
        }
        Ok(())
    }
}
