use tokio::net::UdpSocket;
use tokio::net::UnixStream;

use crate::ipc::StarboardDatagram;

use anyhow::Result;

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
            address: address,
            udp_socket: udp_socket,
            unix_socket: unix_socket,
        }
    }

    pub async fn run(self) -> Result<()> {
        /*tokio::spawn(async move {
            let _: StarboardResult<i8> = async {
                loop {
                    let packet = poll_inputs()?;
                    println!("Test");
                }
            }
            .await?;
        });

        tokio::spawn(async move {
            let _: StarboardResult<i8> = async {
                loop {
                    let packet = poll_inputs()?;
                    println!("Test");
                }
            }
            .await?;
        });*/

        Ok(())
    }
}

fn poll_inputs() -> Result<i8> {
    println!("Inputs polled");
    Ok(1)
}
