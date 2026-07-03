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

use clap::{Arg, ArgAction, Command, value_parser};

use crate::{
    bitmask::Bitmask,
    client::StarboardClient,
    evdev_sb::{SUPPORTED_AXES, SUPPORTED_BUTTONS},
    server::StarboardServerBuilder,
};

// TODO: Implement the args
// Defines all the arguments that 'client' can take in
fn client_args() -> Vec<Arg> {
    vec![]
}

// Defines a command: 'client'
fn client_cmd() -> Command {
    Command::new("client").args(client_args())
}

// TODO: Implement the args
// Defines all the arguments that 'server' can take in
fn server_args() -> Vec<Arg> {
    vec![]
}
// Defines a command: 'server'
fn server_cmd() -> Command {
    Command::new("server").args(server_args())
}

// Defines all the commands
fn starboard_commands() -> Vec<Command> {
    vec![client_cmd(), server_cmd()]
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("starboard")
        .subcommands(starboard_commands())
        .subcommand_required(true)
        .get_matches();

    let server = StarboardServerBuilder::new()
        .enable_buttons(SUPPORTED_BUTTONS)?
        .enable_axes(SUPPORTED_AXES)?
        .set_timeout(15000)
        .build()
        .run();
    Ok(())
}
