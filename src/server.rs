use core::{
    fmt::{self, Display, Formatter},
    time::Duration,
};
use std::collections::{HashMap, HashSet};
use tokio::net::UdpSocket;

use evdev::{AbsoluteAxisCode, KeyCode};

use tokio::{
    select,
    sync::{Mutex, MutexGuard},
    task::JoinSet,
    time::{Interval, interval},
};
use tokio_util::sync::CancellationToken;

use chrono::{DateTime, Local};

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crossterm::event;

use crate::{
    bitmask::Bitmask,
    datagram::{BroadcastPacket, deserialize},
    evdev_sb::{VirtualJoystick, VirtualJoystickBuilder},
    fixed_queue::FixedQueue,
    input::{IntoID, StarboardInputPacket},
    printdbg,
    server_ui::{StarboardServerUI, StarboardSyncUI},
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

impl Display for ControllerState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let string = match self {
            Self::Online => "Online",
            Self::NotResponding => "Not Responding",
        };
        write!(f, "{string}")
    }
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
    pub fn new(
        id: u64,
        name: StarboardString,
        status: ControllerState,
        initial_latency: i64,
    ) -> Self {
        let mut latency = FixedQueue::new();
        latency.push_back(Some(initial_latency));
        Self {
            id,
            name,
            status,
            last_ping: Local::now().timestamp(),
            latency,
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
        // A received packet should always have a value for latency.last()
        let latency = self.latency.last().unwrap();
        write!(f, "{}: {} ({}ms)", self.name, self.status, latency)
    }
}

impl From<BroadcastPacket> for ControllerDiagnostic {
    fn from(value: BroadcastPacket) -> Self {
        Self::new(
            *value.id(),
            *value.name(),
            ControllerState::Online,
            value.latency(),
        )
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
            mutated: AtomicBool::new(false),
            cancellation_token: CancellationToken::new(),
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
    mutated: AtomicBool,
    cancellation_token: CancellationToken,
}

impl StarboardServer {
    // Public facing function to run the server
    pub async fn run(mut self) -> Result<()> {
        if !self.no_ui {
            self.run_ui()?;
        }
        Ok(())
    }

    fn run_ui(&mut self) -> Result<()> {
        let mut ui = StarboardSyncUI::new(
            self.detected_controllers.clone(),
            self.active_controllers.clone(),
            self.cancellation_token.clone(),
        )?;
        while !self.cancellation_token.is_cancelled() {
            self.poll_events(&mut ui)?;
            self.update_ui(&ui)?;
        }
        Ok(())
    }

    fn poll_events(&self, ui: &mut StarboardSyncUI) -> Result<()> {
        while event::poll(Duration::default())? {
            ui.handle_event(event::read()?);
        }
        Ok(())
    }

    fn update_ui(&mut self, ui: &StarboardSyncUI) -> Result<()> {
        if let Ok(true) = self.poll_program_state_change() {
            ui.render()?;
        }
        Ok(())
    }

    fn poll_program_state_change(&mut self) -> Result<bool> {
        Ok(self
            .mutated
            .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
            .or_else(|_| anyhow::bail!("Could not compare and exchange values on `StarboardServer::mutated` atomic boolean"))?)
    }

    // This is the main loop for the server that receives packets and sends them to the input
    // handling
    async fn server_loop(self) -> Result<()> {
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
            let active_controllers = self.active_controllers.lock().await;
            if active_controllers.contains(packet.client_id()) {
                self.handle_packet(&mut virt_joystick, packet)?;
            }
        }
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
    detected_controllers: Arc<Mutex<ControllerMap>>,
) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let sock = UdpSocket::bind(addr).await?;
    let mut raw: [u8; 256] = [0; 256];
    let mut interval = interval(Duration::from_secs(1));
    loop {
        wait_for_controller_update(&detected_controllers, &sock, &mut interval, &mut raw).await?;
    }
}

// Wait for a tick or for a packet to be received, and then update controller diagnostics
// accordingly
async fn wait_for_controller_update(
    detected_controllers: &Arc<Mutex<ControllerMap>>,
    socket: &UdpSocket,
    interval: &mut Interval,
    raw: &mut [u8; 256],
) -> Result<()> {
    tokio::select! {
        _ = interval.tick() => {}
        _ = socket.recv(raw) => {
            let packet: BroadcastPacket = deserialize(Vec::from(&mut *raw))?;
            insert_controller(&mut detected_controllers.lock().await, packet);
        }
    }
    let mut detected_controllers = detected_controllers.lock().await;
    manage_controller_timeouts(&mut detected_controllers);
    detect_controllers(&mut detected_controllers, &socket, raw).await?;
    Ok(())
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
fn insert_controller(
    detected_controllers: &mut MutexGuard<'_, ControllerMap>,
    packet: BroadcastPacket,
) {
    if !detected_controllers.contains_key(packet.id()) {
        let diagnostic: ControllerDiagnostic = packet.into();
        detected_controllers.insert(*diagnostic.id(), diagnostic);
    } else if let Some(diagnostic) = detected_controllers.get_mut(packet.id()) {
        diagnostic.latency.push_back(Some(packet.latency()));
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
