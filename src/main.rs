use evdev::uinput::*;
use evdev::*;
use tokio::sync::{Mutex, MutexGuard};
use tokio::net::UdpSocket;
use std::{io, fs, env};
use std::str::from_utf8;
use std::sync::Arc;
use serde_json::Value;
use serde_json::{json, to_vec};
use libc::input_absinfo;

const KEYS: [KeyCode; 10] = [
            KeyCode::BTN_NORTH,
            KeyCode::BTN_SOUTH,
            KeyCode::BTN_EAST,
            KeyCode::BTN_WEST,
            KeyCode::BTN_THUMBL,
            KeyCode::BTN_THUMBR,
            KeyCode::BTN_TR,
            KeyCode::BTN_TL,
            KeyCode::BTN_START,
            KeyCode::BTN_SELECT
            ];

// This is the function that will receive input data from the Steam Deck and emit an event to the Virtual Device
async fn udp_handling(_device: Arc<Mutex<VirtualDevice>>, socket: Arc<UdpSocket>) {
    let mut buf: [u8; 512] = [0; 512];
    loop {
        let size = socket.recv(&mut buf)
                         .await
                         .unwrap();
        if size > 0 {
            let raw: &str = from_utf8(&buf[..size])
                            .expect("Unable to parse received packet into a utf8 format.\n");
            let parsed: Value = serde_json::from_str(raw)
                                            .expect("Unable to parse utf8 into json format.\n");
            let pressed_keys = &parsed["keys"];
            println!("{}", pressed_keys);
            let abs_values = &parsed["abs_values"];
        }
    }
}

fn get_steam_deck_device() -> Result<Device, &'static str> {
    let dir = fs::read_dir("/dev/input/").expect("/dev/input does not exist.\n");
    for event in dir {
        let path_buf = event.unwrap().path();
        let path = path_buf.to_str().unwrap();
        if !path.starts_with("/dev/input/event") {
            continue;
        }
        let device = Device::open(&path).expect(&format!("Failed to open device at {}\n", path));
        if device.name().unwrap() == "Microsoft X-Box 360 pad 0" {
            return Ok(device);
        }
    }
    Err("Could not access the Steam Deck's input system.")
}

async fn client() {
    let socket = UdpSocket::bind("0.0.0.0:0").await.expect("Could not create a UDP Socket.\n");
    socket.connect("255.255.255.255:9999").await.expect("Could not connect to the local network.\n");
    let device: Device = get_steam_deck_device().expect("Could not access the Steam Deck's input system.");
    while true {
        let key_states: AttributeSet<KeyCode> = device.get_key_state().expect("Failed to get device key states.\n");
        let pressed_keys: Vec<u16> = key_states.iter()
                                                    .map(|k| k.0)
                                                    .collect();
        let abs_states: [input_absinfo; 64] = device.get_abs_state().expect("Failed to get device abs states");
        let abs_values: [i32; 8] = [
                                abs_states[0].value,
                                abs_states[1].value,
                                abs_states[2].value,
                                abs_states[3].value,
                                abs_states[4].value,
                                abs_states[5].value,
                                abs_states[16].value,
                                abs_states[17].value
        ];
        let json = json!({"keys": pressed_keys, "abs_values": abs_values});
        socket.send(to_vec(&json).unwrap().as_slice());
    }
}

async fn server() {
    let mut exit = false;
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
            .with_absolute_axis(&right_z_axis_setup)
            .expect("Could not enable the analog inputs in the right trigger\n")
            .with_absolute_axis(&dpad_x_axis_setup)
            .expect("Could not enable the x axis on the D-pad\n")           
            .with_absolute_axis(&dpad_y_axis_setup)
            .expect("Could not enable the y axis on the D-pad\n")
            .with_keys(&AttributeSet::from_iter(KEYS))
            .expect("Could not enable the gamepad buttons.");
    let device: Arc<Mutex<VirtualDevice>> = Arc::new(Mutex::new(builder.build()
                                                    .expect("Could not build the Virtual Device.\n")));
    let socket: Arc<UdpSocket> = Arc::new(UdpSocket::bind("0.0.0.0:9999").await.expect("Could not create a UDP Socket.\n"));
    tokio::spawn(udp_handling(device.clone(), socket.clone()));
    while !exit {
        let mut device_lock: MutexGuard<VirtualDevice> = device.lock().await;
        let event = [
                    InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_X.0, 32000),
                    InputEvent::new(EventType::KEY.0, KeyCode::BTN_SOUTH.0, 1),
                    InputEvent::new(EventType::SYNCHRONIZATION.0, SynchronizationCode::SYN_REPORT.0, 0),
                    ];
        device_lock.emit(&event);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let event = [
                    InputEvent::new(EventType::ABSOLUTE.0, AbsoluteAxisCode::ABS_X.0, 0),
                    InputEvent::new(EventType::KEY.0, KeyCode::BTN_SOUTH.0, 0),
                    InputEvent::new(EventType::SYNCHRONIZATION.0, SynchronizationCode::SYN_REPORT.0, 0),
                    ];
        device_lock.emit(&event);
    
    }
}
#[tokio::main]
async fn main() {
    let mut is_client = false;
    let mut is_server = false;
    for arg in env::args() {
        if arg == "--client" && is_server == false {
            is_client = true;
            client().await;
        }
        else if arg == "--server" && is_client == false {
            is_server = true;
            server().await;
        }
    }
}