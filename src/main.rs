use evdev::uinput::*;
use evdev::*;
use tokio::sync::Mutex;
use tokio::net::UdpSocket;
use tokio::time::*;
use std::{io, fs, env};
use std::ops::Deref;
use std::str::from_utf8;
use std::sync::{Arc, OnceLock};
use std::collections::HashMap;
use serde_json::{json, to_vec, Value};
use libc::input_absinfo;

static DEBUG_MODE: OnceLock<bool> = OnceLock::new();

macro_rules! debug {
    ($fmt:expr, $($args:tt)*) => {
        if *DEBUG_MODE.get().unwrap() {
            println!(concat!("[DEBUG] ", $fmt), $($args)*);
        }
    };
    ($fmt:expr) => {
        if *DEBUG_MODE.get().unwrap() {
            println!(concat!("[DEBUG] ", $fmt));   
        }
    };
}

#[derive(Default)]
pub struct States {
    key_states: Vec<u16>,
    abs_states: HashMap<AbsoluteAxisCode, i32>
}


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
    
const ABS: [AbsoluteAxisCode; 8] = [
    AbsoluteAxisCode::ABS_X,
    AbsoluteAxisCode::ABS_Y,
    AbsoluteAxisCode::ABS_Z,
    AbsoluteAxisCode::ABS_RX,
    AbsoluteAxisCode::ABS_RY,
    AbsoluteAxisCode::ABS_RZ,
    AbsoluteAxisCode::ABS_HAT0X,
    AbsoluteAxisCode::ABS_HAT0Y
    ];

impl States {
    pub fn new() -> States {
        let mut states: States = Default::default();
        states.key_states = Vec::new();
        states.abs_states = HashMap::new();
        for abs in ABS {
            states.abs_states.insert(abs, 0);
        }
        states
    }
}

// This is the function that will receive input data from the Steam Deck and emit an event to the Virtual Device
// todo: figure out why the latency is longer than the heat death of the universe
async fn udp_handling(device: Arc<Mutex<VirtualDevice>>, socket: Arc<UdpSocket>) {
    let mut states: States = States::new();
    let mut buf: [u8; 512] = [0; 512];
    loop {
        let size = socket.recv(&mut buf)
        .await
        .unwrap();
        if size <= 0 {
            continue;
        }
        
        let raw: &str = from_utf8(&buf[..size])
                        .expect("Unable to parse received packet into a utf8 format.\n");
        let parsed: Value = serde_json::from_str(raw)
                        .expect("Unable to parse utf8 into json format.\n");
        let changed_keys: Vec<u64> = parsed["keys"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|k| k.as_u64().unwrap())
                                .collect();
        let abs_values: Vec<i64> = parsed["abs_values"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|a| a.as_i64().unwrap())
                                .collect();
        let mut device_locked = device.lock().await;
        let mut events: Vec<InputEvent> = Vec::new();
        for i in 0..changed_keys.len() {
            if !states.key_states.contains(&(changed_keys[i] as u16)) {
                states.key_states.push(changed_keys[i] as u16);
                events.push(InputEvent::new(EventType::KEY.0, changed_keys[i].try_into().unwrap(), 1));
            }
        }
        for i in 0..states.key_states.len() {
            if changed_keys.contains(&(states.key_states[i] as u64)) {
                states.key_states.remove(i);
                events.push(InputEvent::new(EventType::KEY.0, changed_keys[i].try_into().unwrap(), 0));
            }
        }
        for i in 0..8 {
            let cached_state = states.abs_states.get_mut(&ABS[i]).unwrap();
            *cached_state = (*cached_state as i64 + abs_values[i]) as i32;
            events.push(InputEvent::new(EventType::ABSOLUTE.0, ABS[i].0, *cached_state));
            
        }
        events.push(InputEvent::new(EventType::SYNCHRONIZATION.0, SynchronizationCode::SYN_REPORT.0, 0));
        let _ = device_locked.emit(events.as_slice());
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

async fn client(framerate: Arc<u64>) {
    let mut states: States = States::new();
    let socket = UdpSocket::bind("0.0.0.0:0").await.expect("Could not create a UDP Socket.\n");
    let _ = socket.set_broadcast(true);
    socket.connect("255.255.255.255:9999").await.expect("Could not connect to the local network.\n");
    let device: Device = get_steam_deck_device().expect("Could not access the Steam Deck's input system.");
    loop {
        let key_states: AttributeSet<KeyCode> = device.get_key_state().expect("Failed to get device key states.\n");
        let pressed_keys: Vec<u16> = key_states.iter()
                                                    .map(|k| k.0)
                                                    .collect();
        let mut changed_keys: Vec<u16> = Vec::new();
        let abs_states: [input_absinfo; 64] = device.get_abs_state().expect("Failed to get device abs states");
        let abs_values: [(AbsoluteAxisCode, i32); 8] = [
            (AbsoluteAxisCode::ABS_X, abs_states[0].value),
            (AbsoluteAxisCode::ABS_Y, abs_states[1].value),
            (AbsoluteAxisCode::ABS_Z, abs_states[2].value),
            (AbsoluteAxisCode::ABS_RX, abs_states[3].value),
            (AbsoluteAxisCode::ABS_RY, abs_states[4].value),
            (AbsoluteAxisCode::ABS_RZ, abs_states[5].value),
            (AbsoluteAxisCode::ABS_HAT0X, abs_states[16].value),
            (AbsoluteAxisCode::ABS_HAT0Y, abs_states[17].value)
        ];
        let mut changed_abs: [i64; 8] = [0; 8];
        let mut has_delta = false;
        if pressed_keys.len() > 0 {
            for i in 0..pressed_keys.len() {
                if states.key_states.contains(&pressed_keys[i]) {
                    break;
                }
                states.key_states.push(pressed_keys[i]);
                changed_keys.push(pressed_keys[i]);
                debug!("Key delta detected on KeyCode: {}", pressed_keys[i]);
            }
        }
        if states.key_states.len() > 0 {
            for i in 0..states.key_states.len() {
                if pressed_keys.contains(&states.key_states[i]) {
                    break;
                }
                changed_keys.push(states.key_states[i]);
                states.key_states.remove(i);
                debug!("Key delta detected on KeyCode: {}", pressed_keys[i]);
            }
        }
        for i in 0..8 {
            let code: &AbsoluteAxisCode  = &ABS[i];
            changed_abs[i] = abs_values[i].1 as i64 - states.abs_states[code] as i64;
            if &changed_abs[i] != &0 {
                has_delta = true;
                debug!("Analog delta detected on code {}: State is {}", code.0, &changed_abs[i]);
            }
            let abs_state = states.abs_states[code];
            let mut_state = states.abs_states.get_mut(code).unwrap();
            *mut_state = (abs_state as i64 + changed_abs[i]) as i32;
        }
        let json = json!({"keys": changed_keys, "abs_values": changed_abs});
        if pressed_keys.len() > 0 || has_delta {
            let _ = socket.send(to_vec(&json).unwrap().as_slice()).await;
        }
        // Synchronize input polling with the framerate of the program so as to not flood the socket with packets
        sleep(Duration::from_millis(1000/framerate.deref())).await;
    }
}

async fn server() {
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
    loop {}
}
#[tokio::main]
async fn main() {
    let mut framerate: Arc<u64> = Arc::new(60);
    let mut is_client = false;
    let mut is_server = false;
    let mut is_debug = false;

    // ARGS:
    // --client             --- Opens starboard in client (controller) mode
    // --server             --- Opens starboard in server (PC) mode
    // --debug              --- Opens starboard in debug mode, which prints extra information to the console
    // --fps=[framerate]    --- Syncs input pulling to the specified framerate
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
    }

    let _ = DEBUG_MODE.set(is_debug);
    debug!("Debug mode is on.");

    if is_client {
        println!("Starting starboard in client mode.");
        client(framerate.clone()).await;
    }

    else if is_server {
        println!("Starting starboard in server mode.");
        server().await;
    }
}
