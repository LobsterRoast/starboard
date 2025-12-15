use evdev::uinput::*;
use evdev::*;

use bincode::config::Configuration;
use bincode::{decode_from_slice, encode_to_vec};

use tokio::net::UdpSocket;

use chrono::{DateTime, FixedOffset, Local};

use crate::debug;
use crate::util::*;

const FF_EFFECTS: [FFEffectCode; 1] = [FFEffectCode::FF_RUMBLE];

async fn get_input_packet(socket: &UdpSocket) -> Option<InputPacket> {
    let mut buf: [u8; 512] = [0; 512];
    loop {
        let size = socket.recv(&mut buf).await.unwrap();
        if size <= 0 {
            continue;
        }
        let conf: Configuration = bincode::config::standard();
        let packet = decode_from_slice::<InputPacket, Configuration>(&buf, conf);
        return match packet {
            Ok(v) => Some(v.0),
            Err(_e) => None,
        };
    }
}

fn parse_timestamp(date: &str) -> Option<DateTime<FixedOffset>> {
    let timestamp = DateTime::parse_from_str(date, "%Y,%m,%d,%H,%M,%S,%3f,%z");
    return match timestamp {
        Ok(v) => Some(v),
        Err(_e) => {
            println!("Unable to decode timestamp.");
            None
        }
    };
}

async fn get_haptic_packet(device: &mut VirtualDevice) -> Option<HapticPacket> {
    loop {
        if let Ok(events) = device.fetch_events() {
            for event in events {
                if event.event_type() != EventType::FORCEFEEDBACK {
                    continue;
                }

                let event_value = f32::from((event.value() as f32 / i32::MAX as f32) * f32::MAX);
                return Some(HapticPacket::new(event_value, get_formatted_time()));
            }
        }
    }
}

// This is the function that will receive input data from the Steam Deck and emit an event to the Virtual Device
fn input(device: &mut VirtualDevice, packet: InputPacket, key_states: &mut u16) {
    let mut events: Vec<InputEvent> = Vec::new();
    if *DEBUG_MODE.get().unwrap() {
        match parse_timestamp(&packet.timestamp) {
            Some(packet_time) => {
                let current_time = Local::now();
                let time_delta = current_time
                    .signed_duration_since(packet_time)
                    .num_milliseconds();
                debug!("Packet received with a latency of {}ms.", time_delta);
            }
            None => {
                debug!("Could not parse a received timestamp.")
            }
        };
    }

    for i in 0..14 {
        let key_bit = BIN_KEYS[i];
        let key_evdev = EVDEV_KEYS[i];
        let key_pressed: u16 = packet.key_states & key_bit;
        let key_pressed_cached: u16 = *key_states & key_bit;
        if key_pressed != key_pressed_cached {
            let event_value = (key_pressed != 0) as i32;
            let event = InputEvent::new(EventType::KEY.0, key_evdev.0, event_value);
            events.push(event);
        }
    }

    *key_states = packet.key_states;

    for i in 0..8 {
        let abs_state = packet.abs_states[i];
        let event = InputEvent::new(EventType::ABSOLUTE.0, EVDEV_AXES[i].0, abs_state);
        events.push(event);
    }

    let synchronization_event = InputEvent::new(
        EventType::SYNCHRONIZATION.0,
        SynchronizationCode::SYN_REPORT.0,
        0,
    );
    events.push(synchronization_event);

    let _ = device.emit(events.as_slice());
}

async fn output(socket: &UdpSocket, packet: HapticPacket) {
    let conf: Configuration = bincode::config::standard();
    let bytes: Vec<u8> = encode_to_vec(packet, conf).expect("Unable to serialize packet.");
    let _ = socket.send(bytes.as_slice()).await;
}

fn get_device() -> VirtualDevice {
    let input_id: InputId = InputId::new(BusType::BUS_VIRTUAL, 0, 0, 0);

    // This is all the info needed to initialize the joysticks and analog trigger inputs
    let axis_info: AbsInfo = AbsInfo::new(0, -32768, 32768, 16, 128, 0);
    let trigger_axis_info: AbsInfo = AbsInfo::new(0, 0, 255, 0, 0, 0);
    let dpad_axis_info: AbsInfo = AbsInfo::new(0, -1, 1, 0, 0, 0);

    let left_x_axis_setup: UinputAbsSetup =
        UinputAbsSetup::new(AbsoluteAxisCode::ABS_X, axis_info.clone());
    let left_y_axis_setup: UinputAbsSetup =
        UinputAbsSetup::new(AbsoluteAxisCode::ABS_Y, axis_info.clone());
    let left_z_axis_setup: UinputAbsSetup =
        UinputAbsSetup::new(AbsoluteAxisCode::ABS_Z, trigger_axis_info.clone());

    let right_x_axis_setup: UinputAbsSetup =
        UinputAbsSetup::new(AbsoluteAxisCode::ABS_RX, axis_info.clone());
    let right_y_axis_setup: UinputAbsSetup =
        UinputAbsSetup::new(AbsoluteAxisCode::ABS_RY, axis_info.clone());
    let right_z_axis_setup: UinputAbsSetup =
        UinputAbsSetup::new(AbsoluteAxisCode::ABS_RZ, trigger_axis_info.clone());

    let dpad_x_axis_setup: UinputAbsSetup =
        UinputAbsSetup::new(AbsoluteAxisCode::ABS_HAT0X, dpad_axis_info.clone());
    let dpad_y_axis_setup: UinputAbsSetup =
        UinputAbsSetup::new(AbsoluteAxisCode::ABS_HAT0Y, dpad_axis_info.clone());

    // We must manually specify which buttons the virtual device will allow.
    // The Steam Deck has 2 joysticks, 2 trackpads, and a lot of buttons.
    // The buttons and joysticks should be simple to implement. For this
    // prototype build, just the joystick axes will be enabled.
    let builder: VirtualDeviceBuilder = VirtualDevice::builder()
        .expect("Could not create a VirtualDeviceBuilder.\n")
        .name("Starboard Virtual Gamepad")
        .input_id(input_id)
        .with_absolute_axis(&left_x_axis_setup)
        .expect("Could not enable the X-axis of the left joystick\n")
        .with_absolute_axis(&left_y_axis_setup)
        .expect("Could not enable the Y-axis of the left joystick\n")
        .with_absolute_axis(&left_z_axis_setup)
        .expect("Could not enable the analog inputs in the left trigger\n")
        .with_absolute_axis(&right_x_axis_setup)
        .expect("Could not enable the X-axis of the right joystick\n")
        .with_absolute_axis(&right_y_axis_setup)
        .expect("Could not enable the Y-axis of the right joystick\n")
        .with_absolute_axis(&right_z_axis_setup) // There couldn't have been a more concise way to do this????
        .expect("Could not enable the analog inputs in the right trigger\n")
        .with_absolute_axis(&dpad_x_axis_setup)
        .expect("Could not enable the x axis on the D-pad\n")
        .with_absolute_axis(&dpad_y_axis_setup) // Just a few more...
        .expect("Could not enable the y axis on the D-pad\n")
        .with_keys(&AttributeSet::from_iter(EVDEV_KEYS))
        .expect("Could not enable the gamepad buttons.\n")
        .with_ff(&AttributeSet::from_iter(FF_EFFECTS))
        .expect("Could not enable haptics.\n")
        .with_ff_effects_max(1); // FINALLY

    builder
        .build()
        .expect("Could not build the Virtual Device.\n")
}

pub async fn server(ip: String, port: u16) {
    let mut device = get_device();
    // 0.0.0.0 is the default ip if another is not specified.
    let bind_ip = get_ip("0.0.0.0".to_string(), ip);
    let address = format!("{}:{}", bind_ip, port);
    let socket = UdpSocket::bind(&address)
        .await
        .expect("Could not create a UDP Socket.\n");
    let mut key_states: u16 = 0;
    debug!("Server socket bound to {}.", address);

    loop {
        // Await both input and haptic packets.
        // When one is received, process it.
        tokio::select! {
            input_packet = get_input_packet(&socket) => {
                if let Some(input_packet) = input_packet {
                    input(&mut device, input_packet, &mut key_states);
                }
            }
            haptic_packet = get_haptic_packet(&mut device) => {
                if let Some(haptic_packet) = haptic_packet {
                    output(&socket, haptic_packet).await;
                }
            }
        }
    }
}

