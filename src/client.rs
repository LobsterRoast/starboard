use crate::datagram::serialize;
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
    // Broadcast a client's presence to server's on the local network
    async fn broadcast_id(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let mut socket = UdpSocket::bind(addr).await?;
        socket.set_broadcast(true);
        let packet = serialize(&self)?;
        loop {
            sleep(Duration::from_secs(15));
            socket.send(&packet).await?;
        }
    }
}
