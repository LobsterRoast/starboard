mod client;
mod ipc;
mod server;
mod systemd;
mod ui;
mod util;
use anyhow::Result;

use nix::unistd::Uid;
use std::env::current_exe;

use crate::client::client;
use crate::server::Server;
use crate::systemd::create_systemd_unit_file;
use crate::ui::*;
use crate::util::*;

async fn manager() {
    let gtk_wrapper = GtkWrapper::new().await.expect("Unable to initialize GTK");
    gtk_wrapper.run();
}

const HELP_TEXT: &'static str = include_str!("../resources/help.txt");

fn print_help_menu() {
    println!("{}", HELP_TEXT);
}

async fn setup() {
    if !Uid::effective().is_root() {
        println!("Error: Setup must be run with sudo/root privileges.");
        return;
    }

    if current_exe().unwrap().to_str().unwrap() != "/usr/local/bin/starboard" {
        println!("Error: Starboard must be installed in /usr/local/bin.");
        return;
    }

    create_systemd_unit_file().await;
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
    return port
        .strip_prefix("--port=")
        .unwrap()
        .parse::<u16>()
        .expect("Port must be a valid integer between 1 and 65535\n");
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
async fn main() -> Result<()> {
    let mut framerate: u64 = 60; // Default framerate to 60
    let mut ip: String = "".to_string();
    let mut port: u16 = 8080;
    let mut ldeadzone = 3000.0;
    let mut rdeadzone = 1500.0;

    let args: Vec<String> = std::env::args()
        .filter(|arg| !(arg.ends_with("starboard") || arg.contains("sudo")))
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

    // This function returns an error if DEBUG_MODE is already set. This can be ignored.
    let _ = DEBUG_MODE.set(false);

    // At some point in the future, this should be refactored to return
    // some sort of error that can be propagated from any of the different
    // commands.
    match cmd {
        &"server" => {
            println!("Starting starboard in server mode.");
            let server = Server::init(ip, port).await;
            server.run().await?;
        }
        &"client" => {
            println!("Starting starboard in client mode.");
            client(framerate, ip, port, ldeadzone, rdeadzone).await;
        }
        &"manager" => manager().await,
        &"help" => print_help_menu(),
        &"setup" => setup().await,
        &&_ => panic!(
            "{} is not a valid command. Run 'starboard help' to see a list of valid commands.",
            cmd
        ),
    }

    Ok(())
}
