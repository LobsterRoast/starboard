use crate::bitmask::Bitmask;
use crate::evdev_sb::{SUPPORTED_AXES, SUPPORTED_BUTTONS, VirtualJoystick, VirtualJoystickBuilder};
use anyhow::Result;

pub struct DummySteamDeck {
    inner: VirtualJoystick,
}

impl DummySteamDeck {
    pub fn new() -> Result<Self> {
        let buttons: Bitmask = SUPPORTED_BUTTONS.iter().map(|button| *button).collect();
        let axes: Bitmask = SUPPORTED_AXES.iter().map(|axis| *axis).collect();
        Ok(Self {
            inner: VirtualJoystickBuilder::new()?
                .enable_buttons_bitmask(buttons)?
                .enable_axes_bitmask(axes)?
                .build("Dummy Steam Deck")?,
        })
    }
}
