mod client;
mod server;
mod util;

use evdev::uinput::*;
use evdev::*;

use tokio::sync::Mutex;
use tokio::net::UdpSocket;
use tokio::time::*;

use std::{fs, env};
use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use std::collections::HashMap;

use bincode::config::Configuration;
use bincode::{encode_to_vec, decode_from_slice};

use libc::input_absinfo;

use chrono::{DateTime, Local, FixedOffset};

use sdl2::controller::{GameController, Button, Axis};
use sdl2::event::EventType as SdlEventType;
use sdl2::event::Event as SdlEvent;
use sdl2::{Sdl, GameControllerSubsystem};

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::*;

use rdev::display_size;

use crate::util::*;
use crate::server::server;

fn get_controller_message(controller: &GameController) -> String {
    let name = controller.name();
    let mapping = controller.mapping();
    return format!("Controller found:\nName: {}\nMapping: {}\n", name, mapping);
}

fn get_contoller(controller_subsystem: GameControllerSubsystem) -> Result<GameController, &'static str> {
    let joystick_count = controller_subsystem.num_joysticks().expect("Unable to count controllers.\n");
    if joystick_count < 1 {
        return Err("No controllers found to connect to.");
    }

    for i in 0..joystick_count {
        if controller_subsystem.is_game_controller(i) {
            return Ok(match controller_subsystem.open(i) {
                Ok(v) => {
                    debug!("{}", get_controller_message(&v));
                    v
                },
                Err(e) => panic!("{}", e)
            })
        }
    }
    
    Err("No valid controllers found to connect to.")
}

// uinput treats the Steam Deck as an analog input. SDL treats it as a binary one.
// This function takes 2 binary inputs and outputs a signed integer to represent them.
fn get_dpad_value(a: bool, b: bool) -> i32 {
    if a & b {
        return 0;  // both are activated
    }

    if !(a | b) {
        return 0;  // neither are activated
    }

    if a {
        return 1;  // a is activated but not b
    }

    return -1;  // b is activated but not a
}

fn get_formatted_time() -> String {
    let dt: DateTime<Local> = Local::now();
    format!("{}", dt.format("%Y,%m,%d,%H,%M,%S,%3f,%z"))
}


async fn client(framerate: Arc<u64>, ip: Arc<String>, port: Arc<u16>) {    
    // The binding isn't really necessary I'm pretty sure but whatever
    let socket = UdpSocket::bind("0.0.0.0:0").await.expect("Could not create a UDP Socket.\n");
    let _ = socket.set_broadcast(true);

    let connect_ip = get_ip("255.255.255.255".to_string(), ip);
    let address = format!("{}:{}", connect_ip, port);

    // Broadcast to all devices on the given port.
    socket.connect(address).await.expect("Could not connect to the local network.\n");

    
    let sdl_context = sdl2::init().expect("Unable to initialize SDL.\n");

    sdl2::hint::set("SDL_HINT_NO_SIGNAL_HANDLERS", "1");
    
    let controller_subsystem = sdl_context.game_controller().expect("Unable to enable SDL Game Controller Subsystem.\n");
    sdl_context.joystick()
        .expect("Could not get joystick subsystem.\n")
        .set_event_state(true);
    
    let mut sdl_event_pump = sdl_context.event_pump().expect("Unable to generate event pump.");

    let controller = match get_contoller(controller_subsystem) {
        Ok(v) => v,
        Err(e) => panic!("{}", e)
    };

    loop {
        for event in sdl_event_pump.poll_iter() {
            match event {
                SdlEvent::ControllerButtonUp {..} => {debug!("Button press")},
                SdlEvent::ControllerButtonDown {..} => {debug!("Button release")},
                SdlEvent::ControllerAxisMotion {..} => {debug!("Axis Motion")},
                _ => {}
            }
        }
        let mut pressed_keys: Vec<Button> = Vec::new();
        for key in SDL_KEYS {
            if controller.button(key) {
                pressed_keys.push(key);
            }
        }

        let mut bitmask: u16 = 0;
        for i in 0..14 {
            if pressed_keys.contains(&SDL_KEYS[i]) {
                debug!("Button pressed: {:?}", &SDL_KEYS[i]);
                bitmask |= BIN_KEYS[i];
            }
        }

        let mut axis_values: [i32; 8] = [0; 8];
        for i in 0..6 {
            axis_values[i] = controller.axis(SDL_AXES[i]) as i32;
        }

        // dpad treated as a binary input by SDL, so this converts it to analog
        axis_values[6] = get_dpad_value(controller.button(Button::DPadRight), 
                                        controller.button(Button::DPadLeft));
        axis_values[7] = get_dpad_value(controller.button(Button::DPadUp), 
                                        controller.button(Button::DPadDown));


        let timestamp = get_formatted_time();

        let conf: Configuration = bincode::config::standard();
        let packet: Packet = Packet::new(bitmask, axis_values, timestamp);
        let bytes: Vec<u8> = encode_to_vec(packet, conf).expect("Unable to serialize packet.");
        let _ = socket.send(bytes.as_slice()).await;
        let _ = socket.send(bytes.as_slice()).await;
    
        // Synchronize input polling with the framerate of the program so as to not flood the socket with packets
        sleep(Duration::from_millis(1000/framerate.deref())).await;
    }
}

#[tokio::main]
async fn main() {
    let mut framerate: Arc<u64> = Arc::new(60);  // Default framerate to 60
    let mut ip: Arc<String> = Arc::new("0".to_string());
    let mut port: Arc<u16> = Arc::new(8080);
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

        match arg {
            "--client" => is_client = !is_server,
            "--server" => is_server = !is_client,
            "--debug"  => is_debug = true,
            _          => println!("Didn't recognize argument '{}'", arg)
        }

        if arg.starts_with("--fps=") {
            framerate = Arc::new(arg.strip_prefix("--fps=").unwrap().parse::<u64>().expect("Could not parse fps into a u16.\n"));
        }

        if arg.starts_with("--ip=") {

            ip = Arc::new(arg.strip_prefix("--ip=").unwrap().to_string());
            let quartets = ip.split('.');
            
            // ip must be in valid ipv4 format (i.e. 255.255.255.255)
            assert_eq!(quartets.clone().count(), 4, "ip must be in 4 quarters (i.e. 255.255.255.255).");
            
            for quartet in quartets {
                let quartet_byte = quartet
                                    .parse::<u8>()
                                    .expect("Unable to parse ip quarter into unsigned 8-bit integer.\n");
                assert!(quartet_byte < 255, "Invalid ip address.");
            }
        }

        if arg.starts_with("--port=") {
            port = Arc::new(arg.strip_prefix("--port=")
                            .unwrap()
                            .parse::<u16>()
                            .expect("Unable to parse ip into unsigned 16-bit integer.\n"));
        }
    }

    let _ = DEBUG_MODE.set(is_debug);
    debug!("Debug mode is on.");

    // The program should not be able to run in both server and client mode.

    if is_client {
        println!("Starting starboard in client mode.");
        client(framerate.clone(), ip.clone(), port.clone()).await;
    }

    else if is_server {
        println!("Starting starboard in server mode.");
        server(ip.clone(), port.clone()).await;
    }
}
