use core::{convert::TryInto, iter::IntoIterator};

use crate::{bitmask::Bitmask, supported_actions::*};
use anyhow::{Result, bail};
use bincode::{Decode, Encode};
use evdev::{AbsInfo, AbsoluteAxisCode, EventType, InputEvent, KeyCode, UinputAbsSetup};

// There are 25 different buttons in SDL3, requiring at least a u32 to cover them all.
#[derive(PartialEq, Eq, Debug, Decode, Encode)]
pub struct StarboardButtonStates {
    pub raw: Bitmask,
}

impl StarboardButtonStates {
    pub fn new() -> Self {
        Self {
            raw: Bitmask::new(BUTTON_COUNT),
        }
    }

    // returns a StarboardInput indicating whether the button at the id-th bit in `raw` is
    // pressed or not
    pub fn get_state(&self, id: u32) -> StarboardInput {
        let value = self.raw.read_bit(id);
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

    // Registers `button` as pressed and packs it ino the `self`
    fn pack_button(&mut self, id: u32, value: bool) -> Result<()> {
        if id >= BUTTON_COUNT {
            bail!("Could not pack button with id {}; id is out of bounds", id);
        }
        self.raw.write_bit(id, value);
        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Decode, Encode)]
pub struct StarboardAxisStates {
    pub axes: [i16; AXIS_COUNT as usize],
}

impl StarboardAxisStates {
    pub fn new() -> Self {
        Self {
            axes: [0; AXIS_COUNT as usize],
        }
    }

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
            .filter(|(id, value)| *value && *id < self.axes.len())
            .for_each(|(id, _)| inputs.push(self.get_state(id as u32)));
        inputs
    }

    // Registers `axis` as holding value `value`
    fn pack_axis(&mut self, id: usize, value: i16) -> Result<()> {
        if id >= self.axes.len() {
            bail!("Could not pack axis with id {}; id is out of bounds", id);
        }
        self.axes[id] = value;
        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Decode, Encode)]
pub struct StarboardInputPacket {
    pub buttons: StarboardButtonStates,
    pub axes: StarboardAxisStates,
    pub id: u64,
}

impl StarboardInputPacket {
    pub fn new(id: u64) -> Self {
        Self {
            buttons: StarboardButtonStates::new(),
            axes: StarboardAxisStates::new(),
            id,
        }
    }

    // Unpack all inputs in the packet into a vector of StarboardInputs
    pub fn unpack(self, button_mask: Bitmask, axis_mask: Bitmask) -> Vec<StarboardInput> {
        let button_states = self.buttons.get_state_with_mask(button_mask);
        let axis_states = self.axes.get_state_with_mask(axis_mask);

        let mut inputs = button_states;
        inputs.extend(axis_states);
        inputs
    }

    // Pack `input` into the packet
    pub fn pack(&mut self, input: StarboardInput) -> Result<()> {
        Ok(match input {
            StarboardInput::Button { id, value } => self.buttons.pack_button(id, value)?,
            StarboardInput::Axis { id, value } => self.axes.pack_axis(id.try_into()?, value)?,
        })
    }

    // pack all inputs in `inputs` into the packet
    pub fn pack_iter<T>(&mut self, inputs: T) -> Result<()>
    where
        T: IntoIterator<Item = StarboardInput>,
    {
        for input in inputs {
            self.pack(input)?;
        }
        Ok(())
    }

    // Returns the ID of the client that sent the packet
    pub fn client_id(&self) -> &u64 {
        &self.id
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
                const EVENT_TYPE: u16 = EventType::ABSOLUTE.0;
                let event_code = FromID::<AbsoluteAxisCode>::from_id(id)?.0;
                InputEvent::new(EVENT_TYPE, event_code, value.into())
            }
            StarboardInput::Button { id, value } => {
                const EVENT_TYPE: u16 = EventType::KEY.0;
                let event_code = FromID::<KeyCode>::from_id(id)?.0;
                InputEvent::new(EVENT_TYPE, event_code, value.into())
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

// Trait to convert a struct into the ID of a Starboard Input
pub trait IntoID {
    fn into_id(self) -> Result<u32>;
}

// Trait to convert a Starboard Input ID into a different struct
pub trait FromID<T> {
    fn from_id(self) -> Result<T>
    where
        T: Sized;
}

impl IntoByte for KeyCode {
    fn into_byte(self) -> Result<u32> {
        Ok((2 as u32).pow(self.into_id()?))
    }
}

impl IntoByte for AbsoluteAxisCode {
    fn into_byte(self) -> Result<u32> {
        Ok((2 as u32).pow(self.into_id()?))
    }
}

impl FromID<UinputAbsSetup> for u32 {
    fn from_id(self) -> Result<UinputAbsSetup>
    where
        UinputAbsSetup: Sized,
    {
        Ok(UinputAbsSetup::new(
            self.from_id()?,
            AXIS_METADATA[self as usize],
        ))
    }
}

impl IntoID for KeyCode {
    fn into_id(self) -> Result<u32> {
        match SUPPORTED_BUTTONS.get_index_of(&self) {
            Some(v) => Ok(v as u32),
            None => bail!("Couldn't convert given KeyCode '{self:?}' into a Starboard ID.",),
        }
    }
}

impl FromID<KeyCode> for u32 {
    fn from_id(self) -> Result<KeyCode>
    where
        KeyCode: Sized,
    {
        let index = self as usize;
        Ok(match SUPPORTED_BUTTONS.get_index(index) {
            Some(v) => *v.0,
            _ => bail!("Couldn't convert given Starboard ID '{self}' into `KeyCode`."),
        })
    }
}

impl IntoID for AbsoluteAxisCode {
    fn into_id(self) -> Result<u32> {
        match SUPPORTED_AXES.get_index_of(&self) {
            Some(v) => Ok(v as u32),
            None => {
                bail!("Couldn't convert given AbsoluteAxisCode '{self:?}' into a Starboard ID.",)
            }
        }
    }
}

impl FromID<AbsoluteAxisCode> for u32 {
    fn from_id(self) -> Result<AbsoluteAxisCode> {
        let index = self as usize;
        Ok(match SUPPORTED_AXES.get_index(index) {
            Some(v) => *v.0,
            _ => bail!("Couldn't convert given Starboard ID '{self}' into `AbsoluteAxisCode`."),
        })
    }
}
