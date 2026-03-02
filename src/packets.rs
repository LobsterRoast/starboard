use bincode::{Decode, Encode, config::Configuration, decode_from_slice, encode_to_vec};

use anyhow::Result;

static BINCODE_CONFIG: Configuration = bincode::config::standard();

#[derive(Default, Debug, PartialEq, bincode::Encode, bincode::Decode)]
pub struct InputPacket {
    pub key_states: u16,
    pub abs_states: [i32; 8],
    pub timestamp: String,
}

#[derive(Default, Debug, PartialEq, bincode::Encode, bincode::Decode)]
pub struct OutputPacket {}

trait Packet {}

impl Packet for InputPacket {}
impl Packet for OutputPacket {}

pub fn deserialize<T: Packet + Decode<()>>(packet_raw: Vec<u8>) -> Result<T> {
    Ok(
        decode_from_slice::<T, Configuration>(packet_raw.as_slice(), BINCODE_CONFIG)
            .map(|tup| tup.0)?,
    )
}

pub fn serialize<T: Packet + Encode>(packet: T) -> Result<Vec<u8>> {
    Ok(encode_to_vec::<T, Configuration>(packet, BINCODE_CONFIG)?)
}
