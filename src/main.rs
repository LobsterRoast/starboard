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

use crate::{client::StarboardClient, server_ui::StarboardServerUI};

#[tokio::main]
async fn main() -> Result<()> {
    let mut ui = StarboardServerUI::new();
    let _ = ui.launch_ui()?;
    let client = StarboardClient::new(0);
    let _ = client.run().await?;
    Ok(())
}
