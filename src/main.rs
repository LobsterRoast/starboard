mod client;
mod server;
mod util;

use std::sync::Arc;
use std::{fs, env};

use libc::input_absinfo;

use crate::util::*;
use crate::server::server;
use crate::client::client;

#[tokio::main]
async fn main() {
    let mut framerate: Arc<u64> = Arc::new(60);  // Default framerate to 60
    let mut ip: Arc<String> = Arc::new("".to_string());
    let mut port: Arc<u16> = Arc::new(8080);
    let mut ldeadzone = 3000.0;
    let mut rdeadzone = 1500.0;
    let mut is_client = false;
    let mut is_server = false;
    let mut is_debug = false;
    
    // ARGS:
    // --client             --- Opens starboard in client (controller) mode
    // --server             --- Opens starboard in server (PC) mode
    // --debug              --- Opens starboard in debug mode, which prints extra information to the console
    // --fps=[framerate]    --- Syncs input polling to the specified framerate
    // --ip=[ipv4 address]  --- custom ipv4; default is the local network
    // --port=[port]        --- custom port; default is 8080

    for arg in env::args() {
        let arg = arg.as_str();

        if !arg.starts_with("--") {
            continue;
        }

        if arg.starts_with("--fps=") {
            framerate = Arc::new(arg.strip_prefix("--fps=").unwrap().parse::<u64>().expect("Could not parse fps into a u16.\n"));
            continue;
        }

        if arg.starts_with("--ip=") {
            let ip_buf = arg.strip_prefix("--ip=").unwrap().to_string();
            let quartets = ip_buf.split('.');
            
            // ip must be in valid ipv4 format (i.e. 255.255.255.255)
            assert_eq!(quartets.clone().count(), 4, "ip must be in 4 quartets (i.e. 255.255.255.255).");
            
            for quartet in quartets {
                let quartet_byte = quartet
                                    .parse::<u8>()
                                    .expect("Unable to parse ip quarter into unsigned 8-bit integer.\n");
                assert!(quartet_byte <= 255, "Invalid ip address.");
            }
            ip = Arc::new(ip_buf);
            continue;
        }

        if arg.starts_with("--port=") {
            port = Arc::new(arg.strip_prefix("--port=")
                            .unwrap()
                            .parse::<u16>()
                            .expect("Unable to parse ip into unsigned 16-bit integer.\n"));
            continue;
        }

        if arg.starts_with("--ldeadzone=") {
            ldeadzone = arg.strip_prefix("--deadzone=")
                            .unwrap()
                            .parse::<f64>()
                            .expect("Unable to parse deadzone argument into 64-bit floating-point number..\n");
            continue;
        }

        if arg.starts_with("--rdeadzone=") {
            rdeadzone = arg.strip_prefix("--rdeadzone=")
                            .unwrap()
                            .parse::<f64>()
                            .expect("Unable to parse deadzone argument into 64-bit floating-point number..\n");
            continue;
        }

        match arg {
            "--client" => is_client = !is_server,
            "--server" => is_server = !is_client,
            "--debug"  => is_debug = true,
            _          => println!("Didn't recognize argument '{}'", arg)
        }

    }

    let _ = DEBUG_MODE.set(is_debug);
    debug!("Debug mode is on.");

    // The program should not be able to run in both server and client mode.

    if is_client {
        println!("Starting starboard in client mode.");
        client(framerate.clone(), ip.clone(), port.clone(), ldeadzone, rdeadzone).await;
    }

    else if is_server {
        println!("Starting starboard in server mode.");
        server(ip.clone(), port.clone()).await;
    }
}
