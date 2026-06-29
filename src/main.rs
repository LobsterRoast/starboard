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

use crate::client::StarboardClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = StarboardClient::new("Test Client", 0)?;
    let _ = client.run().await?;
    Ok(())
}
