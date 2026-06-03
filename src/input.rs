use crate::bitmask::Bitmask;
use anyhow::{Result, bail};
use bincode::{Decode, Encode};
use evdev::{AbsInfo, AbsoluteAxisCode, EventType, InputEvent, KeyCode, UinputAbsSetup};

// There are 25 different buttons in SDL3, requiring at least a u32 to cover them all.
#[derive(PartialEq, Eq, Debug, Decode, Encode)]
pub struct StarboardButtonStates {
    pub raw: u32,
}

impl StarboardButtonStates {
    pub fn new() -> Self {
        Self { raw: 0 }
    }

    // returns a StarboardInput indicating whether the button at the id-th bit in `raw` is
    // pressed or not
    pub fn get_state(&self, id: u32) -> StarboardInput {
        let value = (self.raw & (1 << id)) > 0;
        StarboardInput::Button { id, value }
    }

    // runs get_state for each positive bit in `mask`
    pub fn get_state_with_mask(&self, mask: Bitmask) -> Vec<StarboardInput> {
        let mut inputs: Vec<StarboardInput> = Vec::new();
        mask.into_iter()
            .enumerate()
            .filter(|(_, value)| *value)
            .for_each(|(id, _)| inputs.push(self.get_state(id as u32)));
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
    pub fn get_state_with_mask(&self, mask: Bitmask) -> Vec<StarboardInput> {
        let mut inputs: Vec<StarboardInput> = Vec::new();
        mask.into_iter()
            .enumerate()
            .filter(|(_, value)| *value)
            .for_each(|(id, _)| inputs.push(self.get_state(id as u32)));
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
    pub fn unpack(self, button_mask: Bitmask, axis_mask: Bitmask) -> Vec<StarboardInput> {
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

impl TryInto<InputEvent> for StarboardInput {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<InputEvent> {
        Ok(match self {
            StarboardInput::Axis { id, value } => {
                let event_type = EventType::ABSOLUTE.0;
                let event_code = FromByte::<AbsoluteAxisCode>::from_byte(1 << id)?;
                InputEvent::new(event_type, event_code.0, value.into())
            }
            StarboardInput::Button { id, value } => {
                let event_type = EventType::KEY.0;
                let event_code = FromByte::<KeyCode>::from_byte(1 << id)?;
                InputEvent::new(event_type, event_code.0, value.into())
            }
        })
    }
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

impl FromByte<AbsoluteAxisCode> for u32 {
    fn from_byte(self) -> Result<AbsoluteAxisCode> {
        Ok(match self {
            1 => AbsoluteAxisCode::ABS_X,
            2 => AbsoluteAxisCode::ABS_Y,
            4 => AbsoluteAxisCode::ABS_Z,
            8 => AbsoluteAxisCode::ABS_RX,
            16 => AbsoluteAxisCode::ABS_RY,
            32 => AbsoluteAxisCode::ABS_RZ,
            64 => AbsoluteAxisCode::ABS_HAT0X,
            128 => AbsoluteAxisCode::ABS_HAT0Y,
            _ => bail!("Couldn't convert given u32 into `AbsoluteAxisCode`"),
        })
    }
}

impl IntoByte for AbsoluteAxisCode {
    fn into_byte(self) -> Result<u32> {
        Ok(match self {
            AbsoluteAxisCode::ABS_X => 1,
            AbsoluteAxisCode::ABS_Y => 2,
            AbsoluteAxisCode::ABS_Z => 4,
            AbsoluteAxisCode::ABS_RX => 8,
            AbsoluteAxisCode::ABS_RY => 16,
            AbsoluteAxisCode::ABS_RZ => 32,
            AbsoluteAxisCode::ABS_HAT0X => 64,
            AbsoluteAxisCode::ABS_HAT0Y => 128,
            _ => bail!("Couldn't convert given AbsoluteAxisCode into `u32`"),
        })
    }
}

// Some of the traits of axes (i.e. deadzones) should be customizable
// at runtime rather than at compile time, but evdev requires certain
// values for these traits. We implement this trait for AbsInfo
// instead of just AbsoluteAxisCode so we can bake it default neutral values.
impl FromByte<AbsInfo> for u32 {
    fn from_byte(self) -> Result<AbsInfo>
    where
        AbsInfo: Sized,
    {
        Ok(match self {
            1 => AbsInfo::new(0, -32768, 32767, 16, 128, 0),
            2 => AbsInfo::new(0, -32768, 32767, 16, 128, 0),
            4 => AbsInfo::new(0, 0, 255, 0, 0, 0),
            8 => AbsInfo::new(0, -32768, 32767, 16, 128, 0),
            16 => AbsInfo::new(0, -32768, 32767, 16, 128, 0),
            32 => AbsInfo::new(0, 0, 255, 0, 0, 0),
            64 => AbsInfo::new(0, -1, 1, 0, 0, 2),
            128 => AbsInfo::new(0, -1, 1, 0, 0, 2),
            _ => bail!("Couldn't convert given u32 into `AbsInfo`"),
        })
    }
}

impl FromByte<UinputAbsSetup> for u32 {
    fn from_byte(self) -> Result<UinputAbsSetup>
    where
        UinputAbsSetup: Sized,
    {
        Ok(match self {
            1 => UinputAbsSetup::new(self.from_byte()?, self.from_byte()?),
            2 => UinputAbsSetup::new(self.from_byte()?, self.from_byte()?),
            4 => UinputAbsSetup::new(self.from_byte()?, self.from_byte()?),
            8 => UinputAbsSetup::new(self.from_byte()?, self.from_byte()?),
            16 => UinputAbsSetup::new(self.from_byte()?, self.from_byte()?),
            32 => UinputAbsSetup::new(self.from_byte()?, self.from_byte()?),
            64 => UinputAbsSetup::new(self.from_byte()?, self.from_byte()?),
            128 => UinputAbsSetup::new(self.from_byte()?, self.from_byte()?),
            _ => bail!("Couldn't convert given u32 into `UinputAbsSetup`"),
        })
    }
}
