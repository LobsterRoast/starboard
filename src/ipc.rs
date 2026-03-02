use sdl2::controller::GameController;
use tokio::net::UdpSocket;
use tokio::net::UnixStream;

use anyhow::Result;

pub trait StarboardDatagram: Send + Sync {
    fn recv(&self) -> Result<Option<Vec<u8>>>;
}
pub trait EventPoll {
    fn poll_input(&self) -> u8;
}

impl StarboardDatagram for UdpSocket {
    fn recv(&self) -> Result<Option<Vec<u8>>> {
        let mut buf: [u8; 256] = [0; 256];

        if self.try_recv(&mut buf)? < 1 {
            return Ok(Some(Vec::from(buf)));
        }

        Ok(None)
    }
}
impl StarboardDatagram for UnixStream {
    fn recv(&self) -> Result<Option<Vec<u8>>> {
        let mut buf: [u8; 256] = [0; 256];

        if self.try_read(&mut buf)? > 0 {
            return Ok(Some(Vec::from(buf)));
        }
        Ok(None)
    }
}

impl EventPoll for GameController {
    fn poll_input(&self) -> u8 {
        1
    }
}
