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

static DEBUG_MODE: OnceLock<bool> = OnceLock::new();

// This macro is essentially just a version of println! that will only run if DEBUG_MODE is True.
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

#[derive(bincode::Encode, bincode::Decode, Default)]
pub struct Packet {
    key_states: u16,
    abs_states: [i32; 8],
    timestamp: String
}

#[derive(Default)]
pub struct States {
    key_states: u16,
    abs_states: HashMap<AbsoluteAxisCode, i32>
}

// The client side cannot use evdev to get controller inputs
// This is because evdev cannot read the triggers on the back of the Steam Deck
// Also, for whatever reason, the evdev device for the Steam Deck only seems to work
// with some sort of Steam game open.
const SDL_KEYS: [Button; 14] = [
    Button::Y,
    Button::A,
    Button::B,
    Button::X,
    Button::LeftStick,
    Button::RightStick,
    Button::LeftShoulder,
    Button::RightShoulder,
    Button::Start,
    Button::Guide,
    Button::Paddle1,
    Button::Paddle2,
    Button::Paddle3,
    Button::Paddle4
];

const EVDEV_KEYS: [KeyCode; 14] = [
    KeyCode::BTN_NORTH,
    KeyCode::BTN_SOUTH,
    KeyCode::BTN_EAST,
    KeyCode::BTN_WEST,
    KeyCode::BTN_THUMBL,
    KeyCode::BTN_THUMBR,
    KeyCode::BTN_TL,
    KeyCode::BTN_TR,
    KeyCode::BTN_START,
    KeyCode::BTN_SELECT,
    KeyCode::BTN_TRIGGER_HAPPY1,
    KeyCode::BTN_TRIGGER_HAPPY2,
    KeyCode::BTN_TRIGGER_HAPPY3,
    KeyCode::BTN_TRIGGER_HAPPY4
];

const BIN_KEYS: [u16; 14] = [
    0b0000000000000001,
    0b0000000000000010,
    0b0000000000000100,
    0b0000000000001000,
    0b0000000000010000,
    0b0000000000100000,
    0b0000000001000000,
    0b0000000010000000,
    0b0000000100000000,
    0b0000001000000000,
    0b0000010000000000,
    0b0000100000000000,
    0b0001000000000000,
    0b0010000000000000
];

// Iterable constant array of all the analog values that will be used. Absolute is code for Analog in this case.
// For some reason, the D-Pad is an analog value. ABS_HAT0(X/Y) refers to the D-Pad values.
const SDL_AXES: [Axis; 6] = [
    Axis::LeftX,
    Axis::LeftY,
    Axis::TriggerLeft,
    Axis::RightX,
    Axis::RightY,
    Axis::TriggerRight
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

impl Packet {
    pub fn new(key_states: u16, abs_states: [i32; 8], timestamp: String) -> Packet {
        let mut packet: Packet = Default::default();
        packet.key_states = key_states;
        packet.abs_states = abs_states;
        packet.timestamp = timestamp;
        packet
    }
}

impl States {
    pub fn new() -> States {
        let mut states: States = Default::default();
        states.key_states = 0;
        states.abs_states = HashMap::new();
        for abs in ABS {
            states.abs_states.insert(abs, 0);
        }
        states
    }
}

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
            let event = InputEvent::new(EventType::ABSOLUTE.0, ABS[i].0, abs_state);
            events.push(event);
        }

        let synchronization_event = InputEvent::new(EventType::SYNCHRONIZATION.0, SynchronizationCode::SYN_REPORT.0, 0);
        events.push(synchronization_event);

        let mut device_locked = device.lock().await;
        let _ = device_locked.emit(events.as_slice());
        iteration += 1;
    }
}

fn get_ip(default: String, ip: Arc<String>) -> String {
    if *ip == "0".to_string() {
        return "0.0.0.0".to_string();
    }
    else {
        return default;
    }
}

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

async fn server(ip: Arc<String>, port: Arc<u16>) {
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
