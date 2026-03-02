use tokio::net::UdpSocket;
use tokio::net::UnixStream;
use tokio::task::JoinSet;

use anyhow::Result;
use anyhow::bail;

use std::sync::Arc;

use crate::error::StarboardError;
use crate::ipc::StarboardDatagram;
use crate::util::*;

pub struct Server {
    pub address: String,

    udp_socket: UdpSocket,
    unix_socket: UnixStream,
}

impl Server {
    pub async fn init(ip: String, port: u16) -> Result<Self> {
        // 0.0.0.0 is the default ip if another is not specified.
        let ip = get_ip("0.0.0.0".to_string(), &ip);
        let address = format!("{}:{}", ip, port);

        let udp_socket: UdpSocket = UdpSocket::bind(&address).await?;

        let mut unix_socket: UnixStream = UnixStream::connect("/tmp/starboard.sock").await?;

        Ok(Self {
            address,
            udp_socket,
            unix_socket,
        })
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let mut join_set = JoinSet::new();

        let self_clone = self.clone();
        let input_poll_thread = join_set.spawn(async move { self_clone.input_loop().unwrap() });

        let self_clone = self.clone();
        let output_poll_thread = join_set.spawn(async move { self_clone.output_loop().unwrap() });

        if let Some(res) = join_set.join_next().await {
            res?
        } else {
            bail!(StarboardError::new("Failed to join server threads", 1));
        };

        Ok(())
    }

    fn input_loop(&self) -> Result<()> {
        Ok(())
    }

    fn output_loop(&self) -> Result<()> {
        Ok(())
    }

    fn poll_input_packets(&self) -> Result<()> {
        println!("Inputs polled");
        Ok(())
    }

    fn poll_output_packets(&self) -> Result<()> {
        println!("Inputs polled");
        Ok(())
    }
}
