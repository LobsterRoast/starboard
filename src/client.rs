use tokio::time::*;

use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, OnceLock},
    net::UdpSocket
};

use bincode::{
    config::Configuration,
    {decode_from_slice, encode_to_vec},
};

use sdl2::{
    EventPump, GameControllerSubsystem, Sdl,
    controller::{Axis, Button, GameController},
    event::{Event as SdlEvent, EventType as SdlEventType},
    haptic::Haptic,
};

use chrono::{DateTime, FixedOffset, Local};

use crate::debug;
use crate::util::*;

async fn get_haptic_packet(socket: &UdpSocket) -> Option<HapticPacket> {
    let mut buf: [u8; 128] = [0; 128];
    let size = socket.recv(&mut buf).unwrap();

    if size <= 0 {
        return None;
    }

    let conf: Configuration = bincode::config::standard();
    let packet = decode_from_slice::<HapticPacket, Configuration>(&buf, conf);
    return match packet {
        Ok(v) => Some(v.0),
        Err(_e) => None,
    };
}

async fn output(socket: &UdpSocket, sdl_context: &Sdl, controller: &GameController) {
    let haptic_subsystem = sdl_context
        .haptic()
        .expect("Unable to initialize SDL Haptic Subsystem.\n");
    let mut haptic = haptic_subsystem
        .open_from_joystick_id(controller.instance_id())
        .unwrap();
    let packet = match get_haptic_packet(&socket).await {
        Some(v) => v,
        None => continue,
    };

    haptic.rumble_stop();
    haptic.rumble_play(packet.strength, 1000);
}

async fn input(socket: &UdpSocket, sdl_context: &Sdl, framerate: &u64, ldeadzone: &f64, rdeadzone: &f64) {
    let mut sdl_event_pump = sdl_context
        .event_pump()
        .expect("Unable to generate event pump.");
    let mut bitmask: u16 = 0;
    let mut axis_values: [i32; 8] = [0; 8];
    let key_associations: &HashMap<Button, u16> = get_key_associations();

    handle_events(&mut sdl_event_pump, &mut bitmask, &mut axis_values, &key_associations);
    apply_ldeadzones(&ldeadzone, &mut axis_values);
    apply_rdeadzones(&rdeadzone, &mut axis_values);

    let timestamp = get_formatted_time();
    let conf: Configuration = bincode::config::standard();
    let packet: InputPacket = InputPacket::new(bitmask, axis_values, timestamp);

    let bytes: Vec<u8> = encode_to_vec(packet, conf).expect("Unable to serialize packet.");
    let _ = socket.send(bytes.as_slice());

    // Synchronize input polling with the framerate of the program so as to not flood the socket with packets
    sleep(Duration::from_millis(1000 / framerate)).await;
}
async fn get_udp_socket(ip: String, port: u16) -> Result<UdpSocket, &'static str> {
    // The binding isn't really necessary I'm pretty sure but whatever
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(v) => v,
        Err(_e) => return Err("Could not create a UDP Socket."),
    };

    let _ = socket.set_broadcast(true);
    let connect_ip = get_ip("255.255.255.255".to_string(), ip);
    let address = format!("{}:{}", connect_ip, port);

    debug!("Client socket connected to {}.", &address);

    // Broadcast to all devices on the given port.
    return match socket.connect(address) {
        Ok(v) => Ok(socket),
        Err(_e) => Err("Could not connect to the local network."),
    };
}

fn get_controller_message(controller: &GameController) -> String {
    let name = controller.name();
    let mapping = controller.mapping();
    return format!("Controller found:\nName: {}\nMapping: {}\n", name, mapping);
}

fn get_controller(controller_subsystem: GameControllerSubsystem) -> Result<GameController, &'static str> {
    let joystick_count = controller_subsystem
        .num_joysticks()
        .expect("Unable to count controllers.\n");
    if joystick_count < 1 {
        return Err("No controllers found to connect to.");
    }

    for i in 0..joystick_count {
        if controller_subsystem.is_game_controller(i) {
            return Ok(match controller_subsystem.open(i) {
                Ok(v) => {
                    debug!("{}", get_controller_message(&v));
                    v
                }
                Err(e) => panic!("{}", e),
            });
        }
    }

    Err("No valid controllers found to connect to.")
}

fn get_formatted_time() -> String {
    let dt: DateTime<Local> = Local::now();
    format!("{}", dt.format("%Y,%m,%d,%H,%M,%S,%3f,%z"))
}

fn handle_events(sdl_event_pump: &mut EventPump, pressed_keys: &mut u16, axis_values: &mut [i32; 8], key_associations: &HashMap<Button, u16>,) {
    sdl_event_pump.pump_events();
    for event in sdl_event_pump.poll_iter() {
        match event {
            SdlEvent::ControllerButtonUp { button, .. } => {
                debug!("Button release");
                button_release(button, pressed_keys, &key_associations);
            }
            SdlEvent::ControllerButtonDown { button, .. } => {
                debug!("Button press");
                match button_press(button, pressed_keys, &key_associations) {
                    1 => axis_values[6] = 1,
                    -1 => axis_values[6] = -1,
                    2 => axis_values[7] = 1,
                    -2 => axis_values[7] = -1,
                    _ => {}
                }
            }
            SdlEvent::ControllerAxisMotion { axis, value, .. } => {
                axis_motion(axis, value, axis_values)
            }
            SdlEvent::Quit { .. } => {
                return;
            }
            _ => {}
        }
    }
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
        Axis::TriggerRight => 5,
    };

    axis_values[i] = value.try_into().unwrap();
}

fn apply_ldeadzones(deadzone: &f64, axis_values: &mut [i32; 8]) {
    let left_joystick: Vec<f64> = [axis_values[0], axis_values[1]]
        .into_iter()
        .map(|x| x as f64)
        .collect();

    if (left_joystick[0] * left_joystick[0] + left_joystick[1] * left_joystick[1]).sqrt()
        <= *deadzone
    {
        axis_values[0] = 0;
        axis_values[1] = 0;
    }
}

fn apply_rdeadzones(deadzone: &f64, axis_values: &mut [i32; 8]) {
    let right_joystick: Vec<f64> = [axis_values[3], axis_values[4]]
        .into_iter()
        .map(|x| x as f64)
        .collect();

    if (right_joystick[0] * right_joystick[0] + right_joystick[1] * right_joystick[1]).sqrt()
        <= *deadzone
    {
        axis_values[3] = 0;
        axis_values[4] = 0;
    }
}

pub async fn client(framerate: u64, ip: String, port: u16, ldeadzone: f64, rdeadzone: f64) {
    let socket = get_udp_socket(ip, port).await.unwrap();

    let sdl_context = sdl2::init().expect("Unable to initialize SDL.\n");
    let controller_subsystem = sdl_context
        .game_controller()
        .expect("Unable to initialize SDL Controller Subsystem.\n");
    let controller = match get_controller(controller_subsystem) {
        Ok(v) => v,
        Err(e) => panic!("{}", e)
    };
    
    loop {
        input(&socket, &sdl_context, &framerate, &ldeadzone, &rdeadzone);
        output(&socket, &sdl_context, &controller);
    }
}

