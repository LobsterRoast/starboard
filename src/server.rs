use core::{
    fmt::{self, Display},
    ops::RangeBounds,
    time::Duration,
};
use std::collections::{HashMap, HashSet};
use tokio::net::UdpSocket;

use evdev::{AbsoluteAxisCode, KeyCode};

use tokio::sync::{Mutex, MutexGuard};

use chrono::Local;

use std::sync::Arc;

use crate::{
    bitmask::Bitmask,
    client::StarboardClient,
    datagram::{BroadcastPacket, deserialize, format_addr, serialize},
    evdev_sb::{self, VirtualJoystick, VirtualJoystickBuilder},
    fixed_queue::FixedQueue,
    input::{IntoID, StarboardInput, StarboardInputPacket},
    printdbg,
    server_ui::StarboardServerUI,
    string::StarboardString,
};

use anyhow::Result;

pub type ControllerMap = HashMap<u64, ControllerDiagnostic>;

// Records the current state of a detected controller
#[derive(Debug, Copy, Clone)]
pub enum ControllerState {
    Online,
    NotResponding,
}

// Records various information about a controller
#[derive(Debug, Copy, Clone)]
pub struct ControllerDiagnostic {
    id: u64,
    name: StarboardString,
    status: ControllerState,
    pub last_ping: i64,
    pub latency: FixedQueue<i64, 10>,
}

impl ControllerDiagnostic {
    pub fn new(id: u64, name: StarboardString, status: ControllerState) -> Self {
        Self {
            id,
            name,
            status,
            last_ping: Local::now().timestamp(),
            latency: FixedQueue::new(),
        }
    }

    pub fn id(&self) -> &u64 {
        &self.id
    }

    pub fn name(&self) -> &StarboardString {
        &self.name
    }
}

impl Display for ControllerDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ({:?})", self.name, self.status)
    }
}

impl From<BroadcastPacket> for ControllerDiagnostic {
    fn from(value: BroadcastPacket) -> Self {
        Self::new(*value.id(), *value.name(), ControllerState::Online)
    }
}

pub struct StarboardServerBuilder {
    // A struct to help build a server
    serial_port: u16,
    device_search_port: u16,
    enabled_buttons: Bitmask,
    enabled_axes: Bitmask,
    no_ui: bool,
}

impl StarboardServerBuilder {
    pub fn new(serial_port: u16, device_search_port: u16) -> Self {
        Self {
            serial_port,
            device_search_port,
            enabled_buttons: Bitmask::new(15),
            enabled_axes: Bitmask::new(15),
            no_ui: false,
        }
    }

    // Build the server
    pub fn build(self, name: String) -> StarboardServer {
        let serial_port = self.serial_port;
        let device_search_port = self.device_search_port;
        let enabled_buttons = self.enabled_buttons;
        let enabled_axes = self.enabled_axes;
        let detected_controllers: Arc<Mutex<ControllerMap>> = Arc::new(Mutex::new(HashMap::new()));
        let active_controllers: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));
        let no_ui = self.no_ui;

        StarboardServer {
            serial_port,
            device_search_port,
            enabled_buttons,
            enabled_axes,
            detected_controllers,
            active_controllers,
            name,
            no_ui,
        }
    }

    // Enable `button` on the server
    pub fn enable_button(self, button: KeyCode) -> Result<Self> {
        let mut builder = self;
        let button_code: u32 = button.into_id()?;
        builder.enabled_buttons.write_bit(button_code, true);
        Ok(builder)
    }

    // Enable each button in `buttons` on the server
    pub fn enable_buttons<T>(self, buttons: T) -> Result<Self>
    where
        T: IntoIterator<Item = KeyCode>,
    {
        let mut builder = self;
        for button in buttons {
            builder = builder.enable_button(button)?;
        }
        Ok(builder)
    }

    // Enable `axis` on the server
    pub fn enable_axis(self, axis: AbsoluteAxisCode) -> Result<Self> {
        let mut builder = self;
        let axis_code: u32 = axis.into_id()?;
        builder.enabled_axes.write_bit(axis_code, true);
        Ok(builder)
    }

    // Enable each axes in `axes` on the server
    pub fn enable_axes<T>(self, axes: T) -> Result<Self>
    where
        T: IntoIterator<Item = AbsoluteAxisCode>,
    {
        let mut builder = self;
        for axis in axes {
            builder = builder.enable_axis(axis)?;
        }
        Ok(builder)
    }

    // Only available in debug mode
    pub fn disable_ui(self, no_ui: bool) -> Self {
        let mut builder = self;
        builder.no_ui = no_ui;
        builder
    }
}

pub struct StarboardServer {
    // The server is what will receive input packets from the controller and simulate a virtual
    // joystick on another PC
    serial_port: u16,
    device_search_port: u16,
    enabled_buttons: Bitmask, // Bitmask representing all the enabled buttons on the server
    enabled_axes: Bitmask,    // Bitmask representing all the enabled axes on the server
    detected_controllers: Arc<Mutex<ControllerMap>>,
    active_controllers: Arc<Mutex<HashSet<u64>>>,
    name: String,
    no_ui: bool,
}

impl StarboardServer {
    // Public facing function to run the server
    pub fn run(mut self) -> Result<()> {
        let device_search_port = self.device_search_port.clone();
        let detected_controllers = Arc::clone(&self.detected_controllers);
        let mut ui = StarboardServerUI::new(
            self.detected_controllers.clone(),
            self.active_controllers.clone(),
        );
        let no_ui = self.no_ui;
        tokio::spawn(async move { self.server_loop() });
        tokio::spawn(manage_controllers(device_search_port, detected_controllers));
        if no_ui {
            std::thread::park();
        } else {
            let handle = std::thread::spawn(move || ui.launch_ui());
            let _ = handle.join();
        }
        Ok(())
    }

    // This is the main loop for the server that receives packets and sends them to the input
    // handling
    async fn server_loop(mut self) -> Result<()> {
        let mut buf: [u8; 256] = [0; 256];
        let mut virt_joystick = VirtualJoystickBuilder::new()?
            .enable_buttons_bitmask(self.enabled_buttons)?
            .enable_axes_bitmask(self.enabled_axes)?
            .build(&self.name)?;
        let addr = format!("0.0.0.0:{}", self.serial_port);
        let sock = UdpSocket::bind(addr).await?;
        loop {
            let _ = self.get_packet(&mut buf, &sock).await;
            let raw = Vec::from(&mut buf);
            let packet: StarboardInputPacket = deserialize(raw)?;
            self.handle_packet(&mut virt_joystick, packet)?;
        }
        Ok(())
    }

    // Waits for a packet to be received and writes the data into `buf`
    async fn get_packet(&self, buf: &mut [u8; 256], sock: &UdpSocket) -> Result<()> {
        while sock.recv(buf).await? <= 0 {}
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
async fn manage_controllers(
    port: u16,
    mut detected_controllers: Arc<Mutex<ControllerMap>>,
) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let sock = UdpSocket::bind(addr).await?;
    let mut raw: [u8; 256] = [0; 256];
    loop {
        {
            let mut detected_controllers = detected_controllers.lock().await;
            manage_controller_timeouts(&mut detected_controllers);
            detect_controllers(&mut detected_controllers, &sock, &mut raw).await?;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// For each controller, ensures that it is not timed out
fn manage_controller_timeouts(detected_controllers: &mut MutexGuard<'_, ControllerMap>) {
    for state in detected_controllers.values_mut() {
        replace_if_timed_out(state);
    }
}

// Detect controllers and add them to the detected_controllers list
async fn detect_controllers(
    detected_controllers: &mut MutexGuard<'_, ControllerMap>,
    sock: &UdpSocket,
    raw: &mut [u8],
) -> Result<()> {
    if let Ok(_) = sock.try_recv(raw) {
        printdbg!("Packet received.");
        let packet: BroadcastPacket = deserialize(Vec::from(raw))?;
        insert_controller(detected_controllers, packet);
        Ok(())
    } else {
        Ok(())
    }
}

// Inserts a controller at key `id` if it does not already exist
fn insert_controller(detected_controllers: &mut ControllerMap, packet: BroadcastPacket) {
    if !detected_controllers.contains_key(packet.id()) {
        let diagnostic: ControllerDiagnostic = packet.into();
        detected_controllers.insert(*diagnostic.id(), diagnostic);
    } else if let Some(diagnostic) = detected_controllers.get_mut(packet.id()) {
        diagnostic.last_ping = Local::now().timestamp();
    }
}

// Takes a mutable reference to a controller state, and sets it to `NotResponding` if it has timed
// out
fn replace_if_timed_out(diagnostic: &mut ControllerDiagnostic) {
    if !check_controller_responding(diagnostic) {
        diagnostic.status = ControllerState::NotResponding;
    }
}

// Checks if a controller has sent a ping in less than 15 seconds, and returns true if so
fn check_controller_responding(diagnostic: &ControllerDiagnostic) -> bool {
    Local::now().timestamp() - diagnostic.last_ping < 15
}
