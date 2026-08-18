use std::io::ErrorKind;

use crate::datagram::{BroadcastPacket, serialize};
use crate::evdev_sb::DeviceWrapper;
use crate::input::StarboardInputPacket;
use crate::printdbg;
use crate::string::StarboardString;
use anyhow::Result;
use bincode::{Decode, Encode};
use tokio::net::UdpSocket;
use tokio::time::{Duration, sleep};

// If testing both the client and server on the same device, the loopback address must be used
// instead of the broadcast address.
#[cfg(feature = "loopback")]
static BC_ADDR: &'static str = "127.0.0.1";
#[cfg(not(feature = "loopback"))]
static BC_ADDR: &'static str = "255.255.255.255";

// Since all the info needed for the server to see a client is contained
// in the client struct itself, we can just directly encode and decode
// the client instead of making a separate packet struct
#[derive(Encode, Decode)]
pub struct StarboardClient {
    id: u64,
    name: StarboardString,
    serial_port: u16,
    device_search_port: u16,
}

impl StarboardClient {
    pub fn new(name: &str, serial_port: u16, device_search_port: u16) -> Result<Self> {
        // TODO: Create a system for randomly generating an ID
        Ok(Self {
            id: 0,
            name: StarboardString::try_from(name)?,
            serial_port,
            device_search_port,
        })
    }

    // Run the client loop
    pub async fn run(&self) -> Result<()> {
        let device = DeviceWrapper::get_steam_deck()?;
        let dest_addr = format!("{}:{}", BC_ADDR, self.serial_port);
        let sock = UdpSocket::bind("0.0.0.0:0").await?;
        let _ = sock.set_broadcast(true)?;
        let _ = sock.connect(&dest_addr).await?;
        printdbg!("Serial Socket connected to {}.", dest_addr);
        tokio::spawn(broadcast_presence(
            self.id,
            self.name,
            self.device_search_port,
        ));
        loop {
            let packet = self.create_packet(&device)?;
            let _ = self.send_packet(packet, &sock).await?;
            tokio::time::sleep(Duration::from_millis(16)).await;
        }
    }

    // Creates a `StarboardInputPacket` from the state of a device
    fn create_packet(&self, device: &DeviceWrapper) -> Result<StarboardInputPacket> {
        let mut packet = StarboardInputPacket::new(self.id);
        packet.pack_iter(device.get_button_inputs()?)?;
        packet.pack_iter(device.get_axis_inputs()?)?;
        Ok(packet)
    }

    // Broadcast's `packet` to the local network
    async fn send_packet(&self, packet: StarboardInputPacket, sock: &UdpSocket) -> Result<()> {
        let raw = serialize(packet)?;
        let res = sock.send(&raw).await;
        if let Err(e) = res {
            err_check_connection_refused(e)?;
        }
        Ok(())
    }
}

// Broadcast a client's presence to server's on the local network
async fn broadcast_presence(id: u64, name: StarboardString, port: u16) -> Result<()> {
    let dest_addr = format!("{}:{}", BC_ADDR, port);
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let _ = socket.set_broadcast(true);
    let _ = socket.connect(&dest_addr).await?;
    printdbg!("Device Search Socket connected to {}.", dest_addr);
    let mut packet = BroadcastPacket::new(id, name)?;
    loop {
        // The packet only needs to be created once, but it needs to be updated and serialized on
        // every loop to keep the timestamp up-to-date
        packet.update();
        let packet_raw = serialize(packet)?;
        let res = socket.send(&packet_raw).await;
        if let Err(e) = res {
            err_check_connection_refused(e)?;
        }
        printdbg!("Device Search Packet Sent.");
        let _ = sleep(Duration::from_secs(5)).await;
    }
}

// Connection refused errors should be ignored
fn err_check_connection_refused(err: std::io::Error) -> Result<()> {
    if let ErrorKind::ConnectionRefused = err.kind() {
        Ok(())
    } else {
        Err(err.into())
    }
}
