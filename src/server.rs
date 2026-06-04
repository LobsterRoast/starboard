use std::net::UdpSocket;

use evdev::{AbsoluteAxisCode, KeyCode};

use tokio::time::{Duration, timeout};

use crate::{
    bitmask::Bitmask,
    datagram::{deserialize, serialize},
    input::{StarboardInput, StarboardInputPacket},
    virtual_joystick::{self, VirtualJoystickEvdev},
};

use anyhow::Result;

pub struct StarboardServerBuilder {
    // A struct to help build a server
    ip: [u8; 4],
    port: u16,
    enabled_buttons: Bitmask,
    enabled_axes: Bitmask,
    timeout_ms: u64,
}

impl StarboardServerBuilder {
    fn new() -> Self {
        Self {
            ip: [0; 4],
            port: 8080,
            enabled_buttons: Bitmask::new(14),
            enabled_axes: Bitmask::new(14),
            timeout_ms: 60000,
        }
    }

    // Build the server
    fn build(self) -> StarboardServer {
        use crate::datagram::format_addr;

        // Defaults to returning 0.0.0.0:0
        let address = format_addr(self.ip, self.port);
        let enabled_buttons = self.enabled_buttons;
        let enabled_axes = self.enabled_axes;
        let timeout_ms = Duration::from_millis(self.timeout_ms);

        StarboardServer {
            address,
            enabled_buttons,
            enabled_axes,
            timeout_ms,
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
    fn enable_button(self, button: KeyCode) -> Self {
        let mut builder = self;
        let button_code: u32 = button.0.into();
        builder.enabled_buttons.write_bit(button_code, true);
        builder
    }

    // Enable each button in `buttons` on the server
    fn enable_buttons<T>(self, buttons: T) -> Self
    where
        T: IntoIterator<Item = KeyCode>,
    {
        let mut builder = self;
        for button in buttons {
            builder = builder.enable_button(button);
        }
        builder
    }

    // Enable `axis` on the server
    fn enable_axis(self, axis: AbsoluteAxisCode) -> Self {
        let mut builder = self;
        let axis_code: u32 = axis.0.into();
        builder.enabled_axes.write_bit(axis_code, true);
        builder
    }

    // Enable each axes in `axes` on the server
    fn enable_axes<T>(self, axes: T) -> Self
    where
        T: IntoIterator<Item = AbsoluteAxisCode>,
    {
        let mut builder = self;
        for axis in axes {
            builder = builder.enable_axis(axis);
        }
        builder
    }

    // Sets the period of time after which the server will timeout if a packet is not received
    fn set_timeout(self, timeout_ms: u64) -> Self {
        let mut builder = self;
        builder.timeout_ms = timeout_ms;
        builder
    }
}

pub struct StarboardServer {
    // The server is what will receive input packets from the controller and simulate a virtual
    // joystick on another PC
    address: String,          // i.e. {ip}:{port}
    enabled_buttons: Bitmask, // Bitmask representing all the enabled buttons on the server
    enabled_axes: Bitmask,    // Bitmask representing all the enabled axes on the server
    timeout_ms: Duration,     // The amount of time after which to panic if a packet is not
                              // received
}

impl StarboardServer {
    // This is the main loop for the server that receives packets and sends them to the input
    // handling
    async fn server_loop(
        &self,
        buf: &mut [u8; 256],
        virt_joystick: &mut VirtualJoystickEvdev,
        sock: &UdpSocket,
    ) -> Result<()> {
        loop {
            let _ = timeout(self.timeout_ms, self.get_packet(buf, sock)).await?;
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
        virt_joystick: &mut VirtualJoystickEvdev,
        packet: StarboardInputPacket,
    ) -> Result<()> {
        let inputs = packet.unpack(self.enabled_buttons, self.enabled_axes);
        for input in inputs {
            virt_joystick.send_input(input)?;
        }
        Ok(())
    }
}
