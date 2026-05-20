use bincode::{Decode, Encode};

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
