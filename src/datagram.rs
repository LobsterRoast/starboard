use anyhow::Result;
use bincode::{Decode, Encode, config::Configuration, decode_from_slice, encode_to_vec};

use crate::{input::StarboardInputPacket, string::StarboardString};

static BINCODE_CONFIG: Configuration = bincode::config::standard();

// Return a formatted address (i.e. 255.255.255.255:8080) or a specified default
pub fn format_addr(ip: [u8; 4], port: u16) -> String {
    format!("{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port)
}

// Deserialize binary data into a StarboardInputPacket
pub fn deserialize<T>(raw: Vec<u8>) -> Result<T>
where
    T: Decode<()>,
{
    Ok(decode_from_slice::<T, Configuration>(raw.as_slice(), BINCODE_CONFIG).map(|tup| tup.0)?)
}

// Serialize a packet into binary data
pub fn serialize<T>(packet: T) -> Result<Vec<u8>>
where
    T: Encode,
{
    Ok(encode_to_vec::<T, Configuration>(packet, BINCODE_CONFIG)?)
}

// Packet for a client to broadcast its presence on the network
#[derive(Decode, Encode)]
pub struct BroadcastPacket {
    id: u64,
    name: StarboardString,
    origin: [u8; 4],
    sent_at: i64,
}
