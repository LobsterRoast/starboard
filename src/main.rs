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

#[cfg(feature = "dummy-steam-deck")]
mod dummy_steam_deck;

#[cfg(test)]
mod test;

use anyhow::Result;

use clap::{Arg, ArgMatches, Command};

use crate::{
    client::StarboardClient,
    evdev_sb::{SUPPORTED_AXES, SUPPORTED_BUTTONS},
    server::StarboardServerBuilder,
};

// TODO: Implement the args
// Defines all the arguments that 'client' can take in
fn client_args() -> Vec<Arg> {
    vec![
        Arg::new("serial-port")
            .value_parser(clap::value_parser!(u16))
            .default_value("54321")
            .long("serial-port")
            .help("The port on which the server will send input packets to servers"),
        Arg::new("device-search-port")
            .value_parser(clap::value_parser!(u16))
            .default_value("61000")
            .long("device-search-port")
            .help("The port on which the server will broadcast its presence to servers"),
    ]
}

// Defines a command: 'client'
fn client_cmd() -> Command {
    Command::new("client").args(client_args())
}

// TODO: Implement the args
// Defines all the arguments that 'server' can take in
fn server_args() -> Vec<Arg> {
    vec![
        Arg::new("serial-port")
            .value_parser(clap::value_parser!(u16))
            .default_value("54321")
            .long("serial-port")
            .help("The port on which the server will receive input packets from controllers"),
        Arg::new("device-search-port")
            .value_parser(clap::value_parser!(u16))
            .default_value("61000")
            .long("device-search-port")
            .help("The port on which the server will search for controllers running the starboard client"),
        Arg::new("name")
            .default_value("Starboard Virtual Gamepad")
            .long("name")
            .short('n'),
        #[cfg(feature = "debug")]
        Arg::new("no-ui")
            .action(ArgAction::SetTrue)
            .long("no-ui"),
    ]
}

// Defines a command: 'server'
fn server_cmd() -> Command {
    Command::new("server").args(server_args())
}

// Defines all the commands
fn starboard_commands() -> Vec<Command> {
    vec![client_cmd(), server_cmd()]
}

fn server(subcommand_matches: &ArgMatches) -> Result<()> {
    // Safety of using `unwrap()`: `serial_port` and `device_search_port` both will default if unset
    let serial_port = *(subcommand_matches.get_one::<u16>("serial-port").unwrap());
    let device_search_port = *(subcommand_matches
        .get_one::<u16>("device-search-port")
        .unwrap());
    let name = subcommand_matches
        .get_one::<String>("name")
        .unwrap()
        .to_owned();
    let no_ui = false;
    #[cfg(feature = "debug")]
    let no_ui = subcommand_matches.get_flag("no-ui");
    StarboardServerBuilder::new(serial_port, device_search_port)
        .enable_buttons(SUPPORTED_BUTTONS)?
        .enable_axes(SUPPORTED_AXES)?
        .disable_ui(no_ui)
        .build(name)
        .run()
}

async fn client(subcommand_matches: &ArgMatches) -> Result<()> {
    // Safety of using `unwrap()`: `serial_port` and `device_search_port` both will default if unset
    let serial_port = *(subcommand_matches.get_one::<u16>("serial-port").unwrap());
    let device_search_port = *(subcommand_matches
        .get_one::<u16>("device-search-port")
        .unwrap());
    StarboardClient::new("Starboard Gamepad", serial_port, device_search_port)?
        .run()
        .await
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "dummy-steam-deck")]
    let dummy_steam_deck = crate::dummy_steam_deck::DummySteamDeck::new();

    let matches = Command::new("starboard")
        .subcommands(starboard_commands())
        .subcommand_required(true)
        .get_matches();

    let (subcommand_name, subcommand_matches) = matches.subcommand().unwrap();

    match subcommand_name {
        "server" => server(subcommand_matches)?,
        "client" => client(subcommand_matches).await?,
        &_ => {}
    }
    Ok(())
}
