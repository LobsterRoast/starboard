use crate::bitmask::Bitmask;
use crate::evdev_sb::{SUPPORTED_AXES, SUPPORTED_BUTTONS, VirtualJoystick, VirtualJoystickBuilder};
use anyhow::Result;
use std::time::Duration;

use crate::input::StarboardInput;

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

    pub fn launch(mut self) {
        tokio::spawn(self.create_dummy_inputs());
    }

    async fn create_dummy_inputs(mut self) {
        const SPEED: f64 = 0.2;
        let mut interval = tokio::time::interval(Duration::from_millis(16));
        let mut counter: u16 = 0;
        loop {
            let tick = interval.tick().await;
            counter = (counter + 1) % 360;
            let angle: f64 = counter.into();
            let x = (i16::MAX as f64 * f64::cos(angle * SPEED)) as i16;
            let y = (i16::MAX as f64 * f64::sin(angle * SPEED)) as i16;
            let x_input = StarboardInput::Axis { id: 0, value: x };
            let y_input = StarboardInput::Axis { id: 1, value: y };
            self.inner.send_input(x_input);
            self.inner.send_input(y_input);
            for i in [1, 4, 5, 6, 9, 12] {
                let event = StarboardInput::Button { id: i, value: true };
                self.inner.send_input(event);
            }
            self.inner.sync();
        }
    }
}
