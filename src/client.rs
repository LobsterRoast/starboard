use anyhow::Result;
use tokio::net::UdpSocket;
use tokio::time::{Duration, sleep};

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
        // TODO: Create a method to generate a packet to broadcast
        let packet: [u8; 1] = [0; 1];
        loop {
            sleep(Duration::from_secs(15));
            socket.send(&packet).await?;
        }
    }
}
