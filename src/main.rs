mod client;
mod server;
mod ui;
mod util;

use daemonize::Daemonize;
use std::{env, fs::File};

use crate::client::client;
use crate::server::server;
use crate::ui::*;
use crate::util::*;

fn manager() {
    let gtk_wrapper = GtkWrapper::new().expect("Unable to initialize GTK");
    gtk_wrapper.run();
}

fn print_help_menu() {
    println!(
"COMMANDS:

server  | Runs on the PC that you run your games on and that your Steam Deck connects to.
client  | Runs on your Steam Deck and connects to your PC. 
manager | A GUI to configure server-specific options. Client-specific options are managed by Decky on your Steam Deck.
help | Shows this help menu.

ARGS:

--fps              | Syncs your input polling up to a given FPS. (Client only)
--ip               | Connects to a specific IP. If left blank, it will be retrieved from your config file. Connects to your local network by default.
--port             | Connects to a specific port. If left blank, it will be retrieved from your config file. Connects to port 8080 by default.
-d / --daemonize   | Runs starboard as a daemon.
-lfz / --ldeadzone | Sets the deadzone for your left analog stick. (Client only)
-rdz / --rdeadzone | Sets the deadzone for your right analog stick. (Client only)
-D / --debug       | Runs starboard in debug mode. Prints extra debug information.
")
}

fn get_fps(iter: &mut std::slice::Iter<&str>) -> u64 {
    return iter
        .next()
        .expect("Missing FPS argument")
        .parse::<u64>()
        .expect("Unable to parse FPS argument");
}

fn get_ip(iter: &mut std::slice::Iter<&str>) -> String {
    let ip = iter.next().expect("Missing IP argument");
    let quartets = ip.split('.');

    // ip must be in valid ipv4 format (i.e. 255.255.255.255)
    assert_eq!(
        quartets.clone().count(),
        4,
        "ip must be in 4 quartets (i.e. 255.255.255.255)."
    );

    for quartet in quartets {
        quartet
            .parse::<u8>()
            .expect("Unable to parse ip quarter into unsigned 8-bit integer.\n");
    }

    return ip.to_string();
}

fn get_port(iter: &mut std::slice::Iter<&str>) -> u16 {
    let port = iter.next().expect("Missing port argument");
    let port = port
        .strip_prefix("--port=")
        .unwrap()
        .parse::<u16>()
        .expect("Port must be a valid integer between 1 and 65535\n");
    if port > 65535 {
        return 65535;
    }
    return port;
}

fn get_deadzone(iter: &mut std::slice::Iter<&str>) -> f64 {
    let dz = iter.next().expect("Missing deadzone argument");
    let dz = dz
        .parse::<f64>()
        .expect("Left Deadzone must be a valid decimal between 32,767 and -32,767");
    if dz > 32767.0 {
        return 32767.0;
    }
    if dz < -32767.0 {
        return -32767.0;
    }
    return dz;
}

fn enable_debug_mode() {
    let _ = DEBUG_MODE.set(true);
    debug!("Debug mode is on.");
}

fn daemonize_starboard() {}

#[tokio::main]
async fn main() {
    let mut framerate: u64 = 60; // Default framerate to 60
    let mut ip: String = "".to_string();
    let mut port: u16 = 8080;
    let mut ldeadzone = 3000.0;
    let mut rdeadzone = 1500.0;
    let mut is_client = false;
    let mut is_server = false;
    let mut is_debug = false;
    let mut gtk = true;

    let args: Vec<String> = std::env::args()
        .filter(|arg| !arg.ends_with("starboard"))
        .collect();
    let args: Vec<&str> = args.iter().map(|arg| arg.as_str()).collect();
    let mut iter = args.iter();
    let cmd = iter
        .next()
        .expect("No valid commands given. Run 'starboard help' to see a list of valid commands.");

    while let Some(arg) = iter.next() {
        match arg {
            &"--fps" => framerate = get_fps(&mut iter),
            &"--ip" => ip = get_ip(&mut iter),
            &"--port" => port = get_port(&mut iter),
            &"--ldeadzone" | &"-ldz" => ldeadzone = get_deadzone(&mut iter),
            &"--rdeadzone" | &"-rdz" => rdeadzone = get_deadzone(&mut iter),
            &"--debug" | &"-D" => enable_debug_mode(),
            &"--daemonize" | &"-d" => daemonize_starboard(),
            &&_ => println!(
                "{} is not a recognized argument. Run 'starboard help' to see a list of valid arguments.",
                arg
            ),
        }
    }

    match cmd {
        &"server" => {
            println!("Starting starboard in server mode.");
            server(ip, port).await;
        }
        &"client" => {
            println!("Starting starboard in client mode.");
            client(framerate, ip, port, ldeadzone, rdeadzone).await;
        }
        &"manager" => manager(),
        &"help" => print_help_menu(),

        &&_ => panic!(
            "{} is not a valid command. Run 'starboard help' to see a list of valid commands.",
            cmd
        ),
    }
}
