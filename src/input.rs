// There are 25 different buttons in SDL3, requiring at least a u32 to cover them all.
#[derive(PartialEq, Eq)]
struct StarboardButtonStates {
    raw: u32,
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

#[derive(PartialEq, Eq)]
struct StarboardAxisStates {
    axes: [i32; 6],
}

impl StarboardAxisStates {
    // returns a StarboardInput indicating the state of the id-th axis
    fn get_state(&self, id: u32) -> StarboardInput {
        let value = self.axes[id as usize];
        StarboardInput::Axis { id, value }
    }

    // runs get_state for each positive bit in `mask`
    fn get_state_with_mask(&self, mask: u32) -> Vec<StarboardInput> {
        let mut inputs: Vec<StarboardInput> = Vec::new();
        for id in 0..32 {
            if (mask & (1 << id)) > 0 {
                inputs.push(self.get_state(id));
            }
        }
        inputs
    }
}

#[derive(PartialEq, Eq)]
struct StarboardInputPacket {
    buttons: StarboardButtonStates,
    axes: StarboardAxisStates,
}

impl StarboardInputPacket {
    // Unpack all inputs in the packet into a vector of StarboardInputs
    fn unpack(self, button_mask: u32, axis_mask: u32) -> Vec<StarboardInput> {
        let button_states = self.buttons.get_state_with_mask(button_mask);
        let axis_states = self.axes.get_state_with_mask(axis_mask);

        let mut inputs = button_states;
        inputs.extend(axis_states);
        inputs
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum StarboardInput {
    Axis { id: u32, value: i32 },
    Button { id: u32, value: bool },
}

#[cfg(test)]
mod InputTests {
    use crate::input::{StarboardAxisStates, StarboardButtonStates, StarboardInput};

    const TEST_BUTTON_STATES: StarboardButtonStates = StarboardButtonStates { raw: 13 }; // 0b1101
    const TEST_AXIS_STATES: StarboardAxisStates = StarboardAxisStates {
        axes: [0, 145, 223, 1125, 102, 255],
    };

    #[test]
    fn test_button_get_state() {
        assert_eq!(
            TEST_BUTTON_STATES.get_state(0),
            StarboardInput::Button { id: 0, value: true }
        );

        assert_eq!(
            TEST_BUTTON_STATES.get_state(1),
            StarboardInput::Button {
                id: 1,
                value: false
            }
        );

        assert_eq!(
            TEST_BUTTON_STATES.get_state(2),
            StarboardInput::Button { id: 2, value: true }
        );

        assert_eq!(
            TEST_BUTTON_STATES.get_state(3),
            StarboardInput::Button { id: 3, value: true }
        );
    }
}
