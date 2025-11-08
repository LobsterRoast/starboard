use std::sync::{Arc, OnceLock};

use evdev::uinput::*;
use evdev::*;

use bincode::decode_from_slice;
use bincode::config::Configuration;

use tokio::sync::Mutex;
use tokio::net::UdpSocket;
use tokio::time::*;

use chrono::{DateTime, Local, FixedOffset};

use crate::util::*;
use crate::debug;

async fn get_packet(socket: &Arc<UdpSocket>, buf: &mut [u8; 512]) -> Option<Packet> {
    let size = socket.recv(buf)
                .await
                .unwrap();
    if size <= 0 {
        return None;
    }
    let conf: Configuration = bincode::config::standard();
    let packet = decode_from_slice::<Packet, Configuration>(buf, conf);
    return match packet {
        Ok(v) => Some(v.0),
        Err(_e) => None
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
    }
}

// This is the function that will receive input data from the Steam Deck and emit an event to the Virtual Device
async fn udp_handling(device: Arc<Mutex<VirtualDevice>>, socket: Arc<UdpSocket>) {
    let mut states: States = States::new();
    let mut buf: [u8; 512] = [0; 512];
    let mut iteration: u64 = 0;
    loop {
        let packet: Packet = match get_packet(&socket, &mut buf).await {
            Some(v) => v,
            None => continue
        };
        
        let mut events: Vec<InputEvent> = Vec::new();
        
        if *DEBUG_MODE.get().unwrap() {
            let timestamp = match parse_timestamp(&packet.timestamp) {
                Some(packet_time) => {
                    let current_time = Local::now();
                    let time_delta = current_time.signed_duration_since(packet_time)
                                                    .num_milliseconds();
                    debug!("Packet received with a latency of {}ms.", time_delta);
                },
                None => {
                    debug!("Could not parse a received timestamp.")
                }
            };
        }
        
        for i in 0..14 {
            let key_bit = BIN_KEYS[i];
            let key_evdev = EVDEV_KEYS[i];
            let key_pressed: u16 = packet.key_states & key_bit;
            let key_pressed_cached: u16 = states.key_states & key_bit;
            if key_pressed != key_pressed_cached {
                let event_value = (key_pressed != 0) as i32;
                let event = InputEvent::new(EventType::KEY.0, key_evdev.0, event_value);
                events.push(event);
            }
        }
        
        states.key_states = packet.key_states;
        
        for i in 0..8 {
            let abs_state = packet.abs_states[i];
            let event = InputEvent::new(EventType::ABSOLUTE.0, EVDEV_AXES[i].0, abs_state);
            events.push(event);
        }

        let synchronization_event = InputEvent::new(EventType::SYNCHRONIZATION.0, SynchronizationCode::SYN_REPORT.0, 0);
        events.push(synchronization_event);

        let mut device_locked = device.lock().await;
        let _ = device_locked.emit(events.as_slice());
        iteration += 1;
    }
}

pub async fn server(ip: Arc<String>, port: Arc<u16>) {
    let input_id: InputId = InputId::new(BusType::BUS_VIRTUAL, 0, 0, 0);
    
    // This is all the info needed to initialize the joysticks and analog trigger inputs
    let axis_info: AbsInfo = AbsInfo::new(0, -32768, 32768, 16, 128, 0);
    let trigger_axis_info: AbsInfo = AbsInfo::new(0, 0, 255, 0, 0, 0);
    let dpad_axis_info: AbsInfo = AbsInfo::new(0, -1, 1, 0, 0, 0);
    
    let left_x_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_X, axis_info.clone());
    let left_y_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_Y, axis_info.clone());
    let left_z_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_Z, trigger_axis_info.clone());
    
    let right_x_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RX, axis_info.clone());
    let right_y_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RY, axis_info.clone());
    let right_z_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RZ, trigger_axis_info.clone());

    let dpad_x_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_HAT0X, dpad_axis_info.clone());
    let dpad_y_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_HAT0Y, dpad_axis_info.clone());

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
            .with_absolute_axis(&right_z_axis_setup)  // There couldn't have been a more concise way to do this????
            .expect("Could not enable the analog inputs in the right trigger\n")
            .with_absolute_axis(&dpad_x_axis_setup)
            .expect("Could not enable the x axis on the D-pad\n")           
            .with_absolute_axis(&dpad_y_axis_setup)
            .expect("Could not enable the y axis on the D-pad\n")
            .with_keys(&AttributeSet::from_iter(EVDEV_KEYS))
            .expect("Could not enable the gamepad buttons.");
    let device: Arc<Mutex<VirtualDevice>> = Arc::new(Mutex::new(builder.build()
                                                    .expect("Could not build the Virtual Device.\n")));
    let bind_ip = get_ip("0.0.0.0".to_string(), ip);
    let bind_address = format!("{}:{}", bind_ip, port);
    let socket: Arc<UdpSocket> = Arc::new(UdpSocket::bind(bind_address).await.expect("Could not create a UDP Socket.\n"));
    
    tokio::spawn(udp_handling(device.clone(), socket.clone()));
    loop {
        sleep(Duration::from_secs(100)).await;
    }
}