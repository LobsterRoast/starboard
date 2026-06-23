use core::{ops::RangeBounds, time::Duration};
use std::collections::HashMap;
use std::net::UdpSocket;

use evdev::{AbsoluteAxisCode, KeyCode};

use tokio::time::timeout;

use chrono::Local;

use std::sync::Arc;

use crate::{
    bitmask::Bitmask,
    client::StarboardClient,
    datagram::{deserialize, serialize},
    evdev_sb::{self, VirtualJoystick},
    fixed_queue::FixedQueue,
    input::{StarboardInput, StarboardInputPacket},
    string::StarboardString,
};

use anyhow::Result;

type ControllerMap = HashMap<u64, ControllerState>;

// Records the current state of a detected controller
#[derive(Debug, Copy, Clone)]
enum ControllerState {
    Online(i64),
    NotResponding,
}

// Records various information about a controller
#[derive(Debug, Copy, Clone)]
struct ControllerDiagnostic {
    id: u64,
    name: StarboardString,
    origin: [u8; 4],
    pub last_ping: i64,
    pub latency: FixedQueue<i64, 10>,
}

impl ControllerDiagnostic {
    pub fn new(id: u64, name: StarboardString, origin: [u8; 4]) -> Self {
        Self {
            id,
            name,
            origin,
            last_ping: Local::now().timestamp(),
            latency: FixedQueue::new(),
        }
    }
}

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
        let detected_controllers: Arc<ControllerMap> = Arc::new(HashMap::new());
        let active_controllers: Vec<u64> = Vec::new();

        StarboardServer {
            address,
            enabled_buttons,
            enabled_axes,
            timeout_ms,
            detected_controllers,
            active_controllers,
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
    detected_controllers: Arc<ControllerMap>,
    active_controllers: Vec<u64>,
}

impl StarboardServer {
    // This is the main loop for the server that receives packets and sends them to the input
    // handling
    async fn server_loop(
        &mut self,
        buf: &mut [u8; 256],
        virt_joystick: &mut VirtualJoystick,
        sock: &UdpSocket,
    ) -> Result<()> {
        tokio::spawn(manage_controllers(self.detected_controllers.clone()));
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
        virt_joystick: &mut VirtualJoystick,
        packet: StarboardInputPacket,
    ) -> Result<()> {
        let inputs = packet.unpack(self.enabled_buttons, self.enabled_axes);
        for input in inputs {
            virt_joystick.send_input(input)?;
        }
        Ok(())
    }
}

// Manage controller detection and timeouts
async fn manage_controllers(mut detected_controllers: Arc<ControllerMap>) -> Result<()> {
    let sock = tokio::net::UdpSocket::bind("0.0.0.0:64646").await?;
    let mut raw: [u8; 4] = [0; 4];
    let detected_controllers = Arc::make_mut(&mut detected_controllers);
    loop {
        manage_controller_timeouts(detected_controllers);
        detect_controllers(detected_controllers, &sock, &mut raw).await?;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// For each controller, ensures that it is not timed out
fn manage_controller_timeouts(detected_controllers: &mut ControllerMap) {
    for state in detected_controllers.values_mut() {
        replace_if_timed_out(state);
    }
}

// Detect controllers and add them to the detected_controllers list
async fn detect_controllers(
    detected_controllers: &mut ControllerMap,
    sock: &tokio::net::UdpSocket,
    raw: &mut [u8],
) -> Result<()> {
    if let Ok(_) = sock.try_recv(raw) {
        let id: u64 = deserialize(Vec::from(raw))?;
        insert_controller(detected_controllers, id);
        Ok(())
    } else {
        Ok(())
    }
}

// Inserts a controller at key `id` if it does not already exist
fn insert_controller(detected_controllers: &mut ControllerMap, id: u64) {
    if !detected_controllers.contains_key(&id) {
        detected_controllers.insert(id, ControllerState::Online(Local::now().timestamp()));
    }
}

// Takes a mutable reference to a controller state, and sets it to `NotResponding` if it has timed
// out
fn replace_if_timed_out(state: &mut ControllerState) {
    if check_controller_timed_out(*state) {
        *state = ControllerState::NotResponding;
    }
}

// Checks if a controller has sent a ping in less than 15 seconds, and returns true if so
fn check_controller_timed_out(state: ControllerState) -> bool {
    if let ControllerState::Online(last_ping) = state {
        Local::now().timestamp() - last_ping < 15
    } else {
        false
    }
}
