use tokio::net::UdpSocket;
use tokio::net::UnixStream;
use tokio::task::JoinSet;

use crate::error::StarboardError;
use crate::ipc::StarboardDatagram;

use anyhow::Result;
use anyhow::bail;

use crate::util::*;

pub struct Server {
    pub address: String,

    udp_socket: UdpSocket,
    unix_socket: UnixStream,
}

impl Server {
    pub async fn init(ip: String, port: u16) -> Self {
        // 0.0.0.0 is the default ip if another is not specified.
        let ip = get_ip("0.0.0.0".to_string(), &ip);
        let address = format!("{}:{}", ip, port);

        let udp_socket: UdpSocket = UdpSocket::bind(&address)
            .await
            .expect("Could not create a UDP Socket.\n");

        let mut unix_socket: UnixStream = UnixStream::connect("/tmp/starboard.sock")
            .await
            .expect("Unable to connect to /tmp/starboard.sock");

        Self {
            address,
            udp_socket,
            unix_socket,
        }
    }

    pub async fn run(self) -> Result<()> {
        let mut join_set = JoinSet::new();
        let input_poll_thread = join_set.spawn(async move {});

        let output_poll_thread = join_set.spawn(async move {});

        if let Some(res) = join_set.join_next().await {
        } else {
            bail!(StarboardError::new("Failed to join server threads", 1),);
        }

        Ok(())
    }
}

fn poll_inputs() -> Result<i8> {
    println!("Inputs polled");
    Ok(1)
}
