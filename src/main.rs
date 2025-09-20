use evdev::uinput::*;
use evdev::*;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tokio::net::UdpSocket;
use std::io;
use std::str::from_utf8;
use serde_json::Value;

// This is the function that will receive input data from the Steam Deck and emit an event to the Virtual Device
async fn udp_handling(_device: Arc<Mutex<VirtualDevice>>, socket: Arc<UdpSocket>) {
    let mut buf: [u8; 512] = [0; 512];
    let size = socket.recv(&mut buf).await.unwrap();
    if size > 0 {
        let raw: &str = from_utf8(&buf[..size]).expect("Unable to parse received packet into a utf8 format.\n");
        let parsed: Value = serde_json::from_str(raw).expect("Unable to parse utf8 into json format.\n");
        println!("{}", parsed["Data"]);
    }
}

#[tokio::main]
async fn main() {
    let mut exit = false;
    let input_id: InputId = InputId::new(BusType::BUS_VIRTUAL, 0, 0, 0);
    
    // There will be options to alter the deadzone later.
    // For simplicity and debugging, it is set to 0 right now.
    let mut deadzone = 0;

    let keys = [
                KeyCode::BTN_DPAD_UP, 
                KeyCode::BTN_DPAD_DOWN, 
                KeyCode::BTN_DPAD_LEFT, 
                KeyCode::BTN_DPAD_RIGHT,
                KeyCode::BTN_NORTH,
                KeyCode::BTN_SOUTH,
                KeyCode::BTN_EAST,
                KeyCode::BTN_WEST,
                KeyCode::BTN_THUMBL,
                KeyCode::BTN_THUMBR,
                KeyCode::BTN_TR,
                KeyCode::BTN_TL,
                KeyCode::BTN_TR2,
                KeyCode::BTN_TL2,
                KeyCode::BTN_TRIGGER_HAPPY1,
                KeyCode::BTN_TRIGGER_HAPPY2,
                KeyCode::BTN_TRIGGER_HAPPY3,
                KeyCode::BTN_TRIGGER_HAPPY4,
                KeyCode::BTN_MODE,
                KeyCode::BTN_START,
                KeyCode::BTN_SELECT
    ];
    // This is all the info needed to initialize the joysticks
    let axis_info: AbsInfo = AbsInfo::new(0, -32768, 32768, 0, deadzone, 0);
    let left_x_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_X, axis_info.clone());
    let left_y_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_Y, axis_info.clone());
    let right_x_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RX, axis_info.clone());
    let right_y_axis_setup: UinputAbsSetup = UinputAbsSetup::new(AbsoluteAxisCode::ABS_RY, axis_info.clone());
    
    // We must manually specify which buttons the virtual device will allow.
    // The Steam Deck has 2 joysticks, 2 trackpads, and a lot of buttons.
    // The buttons and joysticks should be simple to implement. For this
    // prototype build, just the joystick axes will be enabled.
    let builder: VirtualDeviceBuilder = VirtualDevice::builder()
            .expect("Could not create a VirtualDeviceBuilder.")
            .name("Starboard Virtual Gamepad")
            .input_id(input_id)
            .with_absolute_axis(&left_x_axis_setup)
            .expect("Could not enable the X-axis of the left joystick")
            .with_absolute_axis(&left_y_axis_setup)
            .expect("Could not enable the Y-axis of the left joystick")
            .with_absolute_axis(&right_x_axis_setup)
            .expect("Could not enable the X-axis of the right joystick")
            .with_absolute_axis(&right_y_axis_setup)
            .expect("Could not enable the Y-axis of the right joystick")
            .with_keys(&AttributeSet::from_iter(keys))
            .expect("Could not enable the gamepad buttons.");
    let device: Arc<Mutex<VirtualDevice>> = Arc::new(Mutex::new(builder.build()
                                                    .expect("Could not build the Virtual Device.")));
    let socket: Arc<UdpSocket> = Arc::new(UdpSocket::bind("0.0.0.0:9999").await.expect("Could not create a UDP Socket."));
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
