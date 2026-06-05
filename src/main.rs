mod bitmask;
mod client;
mod datagram;
mod evdev_sb;
mod input;
mod server;
#[cfg(test)]
mod test;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
