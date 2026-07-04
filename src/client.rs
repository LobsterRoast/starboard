use core::iter::FromIterator;

use crate::datagram::{BroadcastPacket, serialize};
use crate::evdev_sb::DeviceWrapper;
use crate::input::{IntoByte, StarboardInputPacket};
use crate::string::StarboardString;
use anyhow::Result;
use bincode::{Decode, Encode};
use tokio::net::UdpSocket;
use tokio::time::{Duration, sleep};

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
            device_search_port
        })
    }

    // Run the client loop
    pub async fn run(&self) -> Result<()> {
        tokio::spawn(broadcast_presence(self.id, self.name, self.device_search_port));
        let device = DeviceWrapper::get_steam_deck()?;
        let addr = format!("255.255.255.255:{}", self.serial_port);
        let mut sock = UdpSocket::bind(&addr).await?;
        let _ = sock.connect(addr).await?;
        loop {
            let packet = self.create_packet(&device)?;
            let _ = self.send_packet(packet, &sock).await?;
        }
        Ok(())
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
        sock.send(&raw).await?;
        Ok(())
    }
}

// Broadcast a client's presence to server's on the local network
async fn broadcast_presence(id: u64, name: StarboardString, port: u16) -> Result<()> {
    let addr = format!("255.255.255.255:{}", port);
    let mut socket = UdpSocket::bind(addr).await?;
    let _ = socket.set_broadcast(true);
    let packet = BroadcastPacket::new(id, name)?;
    let packet = serialize(packet)?;
    loop {
        socket.send(&packet).await?;
        let _ = sleep(Duration::from_secs(15)).await;
    }
}
