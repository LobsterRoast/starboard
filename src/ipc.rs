use tokio::net::UdpSocket;
use tokio::net::UnixStream;

pub trait StarboardDatagram {}

impl StarboardDatagram for UdpSocket {}
impl StarboardDatagram for UnixStream {}
