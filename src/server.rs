use sdl2::GameControllerSubsystem;
use sdl2::Sdl;
use sdl2::controller::GameController;
use tokio::net::UdpSocket;
use tokio::net::UnixStream;
use tokio::task::JoinSet;

use anyhow::{Result, anyhow, bail};

use std::sync::Arc;

use crate::error::StarboardError;
use crate::ipc::EventPoll;
use crate::ipc::StarboardDatagram;
use crate::packets::*;
use crate::util::get_ip;

pub struct Server {
    pub address: String,

    sdl_context: Sdl,
    controller_subsystem: GameControllerSubsystem,
    controller: GameController,

    udp_socket: UdpSocket,
    unix_socket: UnixStream,
}

async fn poll_input_packets_server(src: &dyn EventPoll) -> Result<Option<InputPacket>> {
    println!("Inputs polled");
    let raw_event_data = src.poll_input();
    /*
    Some(serialize::<InputPacket>(raw_event_data)?)
    */
    Ok(None)
}

async fn poll_output_packets_server(src: &dyn StarboardDatagram) -> Result<Option<InputPacket>> {
    println!("Outputs polled");
    if let Some(raw_packet) = src.recv()? {
        return Ok(Some(deserialize::<InputPacket>(raw_packet)?));
    }
    Ok(None)
}

impl Server {
    pub async fn init(ip: String, port: u16) -> Result<Self> {
        // 0.0.0.0 is the default ip if another is not specified.
        let ip = get_ip("0.0.0.0".to_string(), &ip);
        let address = format!("{}:{}", ip, port);

        let sdl_context = sdl2::init().map_err(|e| anyhow!("Failed to initialize SDL: {e}"))?;

        let controller_subsystem = sdl_context
            .game_controller()
            .map_err(|e| anyhow!("Failed to initialize SDL Controller Subsystem: {e}"))?;

        let controller = Self::find_controller(&controller_subsystem)?;

        let udp_socket: UdpSocket = UdpSocket::bind(&address).await?;

        let mut unix_socket: UnixStream = UnixStream::connect("/tmp/starboard.sock").await?;

        Ok(Self {
            address,
            sdl_context,
            controller_subsystem,
            controller,
            udp_socket,
            unix_socket,
        })
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let mut join_set = JoinSet::new();

        /*let self_clone = self.clone();
        let _ = join_set.spawn(async move { self_clone.input_loop_server().await.unwrap() });

        let self_clone = self.clone();
        let _ = join_set.spawn(async move { self_clone.output_loop_server().await.unwrap() });*/

        if let Some(res) = join_set.join_next().await {
            join_set.abort_all();
            res?
        } else {
            bail!(StarboardError::new(1, "Failed to join server threads"));
        };

        Ok(())
    }

    fn find_controller(subsystem: &GameControllerSubsystem) -> Result<GameController> {
        let num_joysticks: u32 = subsystem
            .num_joysticks()
            .map_err(|e| anyhow!("Failed to determine number of joysticks: {e}"))?;
        if num_joysticks < 1 {
            bail!(StarboardError::new(2, "No joysticks detected"));
        }

        let controller_index = Self::iterate_controller_indices(&subsystem, num_joysticks)?;

        Ok(subsystem.open(controller_index)?)
    }

    fn iterate_controller_indices(
        subsystem: &GameControllerSubsystem,
        upper_index_bounds: u32,
    ) -> Result<u32> {
        for i in 0..upper_index_bounds {
            if subsystem.is_game_controller(i) {
                return Ok(i);
            }
        }
        bail!(StarboardError::new(
            2,
            "Failed to find appropriate controller"
        ));
    }

    async fn input_loop_server(&self) -> Result<()> {
        loop {
            if let Some(packet) = poll_input_packets_server(&self.controller).await? {}
        }
    }

    async fn output_loop_server(&self) -> Result<()> {
        loop {
            if let Some(packet) = poll_output_packets_server(&self.udp_socket).await? {}
        }
    }
}
