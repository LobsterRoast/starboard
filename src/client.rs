use crate::datagram::serialize;
use crate::input::StarboardInputPacket;
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
    port: u16,
}

impl StarboardClient {
    // Broadcast's `packet` to the local network
    async fn send_packet(&self, packet: StarboardInputPacket, sock: &UdpSocket) -> Result<()> {
        let raw = serialize(packet)?;
        sock.send(&raw).await?;
        Ok(())
    }

    // Broadcast a client's presence to server's on the local network
    async fn broadcast_id(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let mut socket = UdpSocket::bind(addr).await?;
        let _ = socket.set_broadcast(true);
        let packet = serialize(&self)?;
        loop {
            socket.send(&packet).await?;
            let _ = sleep(Duration::from_secs(15)).await;
        }
    }
}
