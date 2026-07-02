mod bitmask;
mod client;
mod datagram;
mod debug;
mod evdev_sb;
mod fixed_queue;
mod input;
mod server;
mod server_ui;
mod string;
#[cfg(test)]
mod test;

use anyhow::Result;

use crate::{
    bitmask::Bitmask,
    client::StarboardClient,
    evdev_sb::{SUPPORTED_AXES, SUPPORTED_BUTTONS},
    server::StarboardServerBuilder,
};

#[tokio::main]
async fn main() -> Result<()> {
    let server = StarboardServerBuilder::new()
        .enable_buttons(SUPPORTED_BUTTONS)?
        .enable_axes(SUPPORTED_AXES)?
        .set_timeout(15000)
        .build()
        .run();
    Ok(())
}
