use crate::bitmask::Bitmask;
use crate::evdev_sb::{SUPPORTED_AXES, SUPPORTED_BUTTONS, VirtualJoystick, VirtualJoystickBuilder};

pub struct DummySteamDeck {
    inner: VirtualJoystick,
}

impl DummySteamDeck {
    pub fn new() -> Self {
        let buttons: Bitmask = SUPPORTED_BUTTONS.iter().map(|button| *button).collect();
        let axes: Bitmask = SUPPORTED_AXES.iter().map(|axis| *axis).collect();
        Self {
            inner: VirtualJoystickBuilder::new()
                .unwrap()
                .enable_buttons_bitmask(buttons)
                .unwrap()
                .enable_axes_bitmask(axes)
                .unwrap()
                .build("Dummy Steam Deck")
                .unwrap(),
        }
    }
}
