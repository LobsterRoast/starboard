mod bitmask;
mod client;
mod datagram;
mod debug;
mod evdev_sb;
mod input;
mod server;
mod server_ui;
#[cfg(test)]
mod test;

use anyhow::Result;

use crate::client::StarboardClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = StarboardClient::new(0);
    let _ = client.run().await?;
    Ok(())
}
