use tokio::time::*;
use tokio::net::UdpSocket;

use std::ops::Deref;
use std::sync::{Arc, OnceLock};
use std::collections::HashMap;

use bincode::config::Configuration;
use bincode::encode_to_vec;

use sdl2::controller::{GameController, Button, Axis};
use sdl2::event::EventType as SdlEventType;
use sdl2::event::Event as SdlEvent;
use sdl2::{Sdl, GameControllerSubsystem};

use chrono::{DateTime, Local, FixedOffset};

use crate::util::*;
use crate::debug;

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

// Returns a code if the button is D-Pad press (which requires special logic to convert to an analog value).
fn button_press(button: Button, bitmask: &mut u16, key_associations: &HashMap<Button, u16>) -> i8 {
    match button {
        Button::DPadUp => return 1,
        Button::DPadDown => return -1,
        Button::DPadRight => return 2,
        Button::DPadLeft => return -2,
        _ => {}
    }

    if let Some(bin) = key_associations.get(&button) {
        *bitmask += bin;
    } else {
        return 0;
    }

    0
}

fn button_release(button: Button, bitmask: &mut u16, key_associations: &HashMap<Button, u16>) -> i8 {
    if let Some(bin) = key_associations.get(&button) {
        *bitmask -= bin;
    } else {
        return 0;
    }

    0
}

fn axis_motion(axis: Axis, value: i16, axis_values: &mut [i32; 8]) {
    let i = match axis {
            Axis::LeftX => 0,
            Axis::LeftY => 1,
            Axis::TriggerLeft => 2,
            Axis::RightX => 3,
            Axis::RightY => 4,
            Axis::TriggerRight => 5
    };

    axis_values[i] = value.try_into().unwrap();
}

pub async fn client(framerate: Arc<u64>, ip: Arc<String>, port: Arc<u16>) {    
    // The binding isn't really necessary I'm pretty sure but whatever
    let socket = UdpSocket::bind("0.0.0.0:0").await.expect("Could not create a UDP Socket.\n");
    let _ = socket.set_broadcast(true);
    let connect_ip = get_ip("255.255.255.255".to_string(), ip);
    let address = format!("{}:{}", connect_ip, port);

    debug!("Client socket connected to {}.", &address);

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

    let mut bitmask: u16 = 0;
    let key_associations: &HashMap<Button, u16> = get_key_associations();
    
    loop {
        let mut axis_values: [i32; 8] = [0; 8];

        sdl_event_pump.pump_events();
        for event in sdl_event_pump.poll_iter() {
            match event {
                SdlEvent::ControllerButtonUp { button, ..} => {
                    debug!("Button release");
                    button_release(button, &mut bitmask, &key_associations);
                },
                SdlEvent::ControllerButtonDown { button, ..} => {
                    debug!("Button press");
                    match button_press(button, &mut bitmask, &key_associations) {
                        1  => { axis_values[6] = 1  },
                        -1 => { axis_values[6] = -1 },
                        2  => { axis_values[7] = 1  },
                        -2 => { axis_values[7] = -1 },
                        _  => {}
                    }
                },
                SdlEvent::ControllerAxisMotion { axis, value, ..} => {
                    axis_motion(axis, value, &mut axis_values)
                },
                SdlEvent::Quit {..} => {
                    return;
                },
                _ => {}
            }
        }

        let timestamp = get_formatted_time();
        let conf: Configuration = bincode::config::standard();
        let packet: Packet = Packet::new(bitmask, axis_values, timestamp);
        let bytes: Vec<u8> = encode_to_vec(packet, conf).expect("Unable to serialize packet.");
        let _ = socket.send(bytes.as_slice()).await;
    
        // Synchronize input polling with the framerate of the program so as to not flood the socket with packets
        sleep(Duration::from_millis(1000/framerate.deref())).await;
    }
}