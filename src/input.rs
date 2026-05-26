use anyhow::{Result, bail};
use bincode::{Decode, Encode};
use evdev::KeyCode;

// There are 25 different buttons in SDL3, requiring at least a u32 to cover them all.
#[derive(PartialEq, Eq, Debug, Decode, Encode)]
pub struct StarboardButtonStates {
    pub raw: u32,
}

impl StarboardButtonStates {
    pub fn new() -> Self {
        Self { raw: 0 }
    }

    // delta() is symmetric.
    // That is, A.delta(&B) == B.delta(&A).
    pub fn delta(&self, other: &StarboardButtonStates) -> u32 {
        self.raw ^ other.raw
    }

    // returns a StarboardInput indicating whether the button at the id-th bit in `raw` is
    // pressed or not
    pub fn get_state(&self, id: u32) -> StarboardInput {
        let value = (self.raw & (1 << id)) > 0;
        StarboardInput::Button { id, value }
    }

    // runs get_state for each positive bit in `mask`
    pub fn get_state_with_mask(&self, mask: u32) -> Vec<StarboardInput> {
        let mut inputs: Vec<StarboardInput> = Vec::new();
        for id in 0..32 {
            if (mask & (1 << id)) > 0 {
                inputs.push(self.get_state(id));
            }
        }
        inputs
    }
}

#[derive(PartialEq, Eq, Debug, Decode, Encode)]
pub struct StarboardAxisStates {
    pub axes: [i16; 6],
}

impl StarboardAxisStates {
    // returns a StarboardInput indicating the state of the id-th axis
    pub fn get_state(&self, id: u32) -> StarboardInput {
        let value = self.axes[id as usize];
        StarboardInput::Axis { id, value }
    }

    // runs get_state for each positive bit in `mask`
    pub fn get_state_with_mask(&self, mask: u32) -> Vec<StarboardInput> {
        let mut inputs: Vec<StarboardInput> = Vec::new();
        let upper = self.axes.len();
        for id in 0..upper {
            let id = id as u32;
            if (mask & (1 << id)) > 0 {
                inputs.push(self.get_state(id));
            }
        }
        inputs
    }
}

#[derive(PartialEq, Eq, Debug, Decode, Encode)]
pub struct StarboardInputPacket {
    pub buttons: StarboardButtonStates,
    pub axes: StarboardAxisStates,
}

impl StarboardInputPacket {
    // Unpack all inputs in the packet into a vector of StarboardInputs
    pub fn unpack(self, button_mask: u32, axis_mask: u32) -> Vec<StarboardInput> {
        let button_states = self.buttons.get_state_with_mask(button_mask);
        let axis_states = self.axes.get_state_with_mask(axis_mask);

        let mut inputs = button_states;
        inputs.extend(axis_states);
        inputs
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum StarboardInput {
    Axis { id: u32, value: i16 },
    Button { id: u32, value: bool },
}

// Trait to convert a foreign library's input type into a relevant bitmask
pub trait IntoByte {
    fn into_byte(self) -> Result<u32>;
}

// Trait to convert a bitmask into a foreign library's input type
pub trait FromByte<T> {
    fn from_byte(self) -> Result<T>
    where
        T: Sized;
}

impl FromByte<KeyCode> for u32 {
    fn from_byte(self) -> Result<KeyCode>
    where
        KeyCode: Sized,
    {
        Ok(match self {
            1 => KeyCode::BTN_NORTH,
            2 => KeyCode::BTN_SOUTH,
            4 => KeyCode::BTN_EAST,
            8 => KeyCode::BTN_WEST,
            16 => KeyCode::BTN_THUMBL,
            32 => KeyCode::BTN_THUMBR,
            64 => KeyCode::BTN_TL,
            128 => KeyCode::BTN_TR,
            256 => KeyCode::BTN_START,
            512 => KeyCode::BTN_SELECT,
            1024 => KeyCode::BTN_TRIGGER_HAPPY1,
            2048 => KeyCode::BTN_TRIGGER_HAPPY2,
            4096 => KeyCode::BTN_TRIGGER_HAPPY3,
            8192 => KeyCode::BTN_TRIGGER_HAPPY4,
            _ => bail!("Couldn't convert given KeyCode into `u32`"),
        })
    }
}

impl IntoByte for KeyCode {
    fn into_byte(self) -> Result<u32> {
        Ok(match self {
            KeyCode::BTN_NORTH => 1,
            KeyCode::BTN_SOUTH => 2,
            KeyCode::BTN_EAST => 4,
            KeyCode::BTN_WEST => 8,
            KeyCode::BTN_THUMBL => 16,
            KeyCode::BTN_THUMBR => 32,
            KeyCode::BTN_TL => 64,
            KeyCode::BTN_TR => 128,
            KeyCode::BTN_START => 256,
            KeyCode::BTN_SELECT => 512,
            KeyCode::BTN_TRIGGER_HAPPY1 => 1024,
            KeyCode::BTN_TRIGGER_HAPPY2 => 2048,
            KeyCode::BTN_TRIGGER_HAPPY3 => 4096,
            KeyCode::BTN_TRIGGER_HAPPY4 => 8192,
            _ => bail!("Couldn't covert given u32 into `KeyCode`"),
        })
    }
}
