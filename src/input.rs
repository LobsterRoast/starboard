#[derive(PartialEq, Eq)]
struct StarboardButtonStates {
    // There are 25 different buttons in SDL3, requiring at least a u32 to cover them all.
    raw: u32,
}

impl StarboardButtonStates {
    pub fn new() -> Self {
        Self { raw: 0 }
    }

    pub fn delta(&self, other: &StarboardButtonStates) -> u32 {
        // delta() is symmetric.
        // That is, A.delta(&B) == B.delta(&A).

        self.raw ^ other.raw
    }

    pub fn get_state(&self, id: u32) -> StarboardInput {
        // returns a StarboardInput indicating whether the button at the id-th bit in `raw` is
        // pressed or not
        let value = (self.raw & (1 << id)) > 0;
        StarboardInput::Button { id, value }
    }

    pub fn get_state_with_mask(&self, mask: u32) -> Vec<StarboardInput> {
        // runs get_state for each positive bit in `mask`
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
    fn get_state(&self, id: u32) -> StarboardInput {
        // returns a StarboardInput indicating the state of the id-th axis
        let value = self.axes[id as usize];
        StarboardInput::Axis { id, value }
    }

    fn get_state_with_mask(&self, mask: u32) -> Vec<StarboardInput> {
        // runs get_state for each positive bit in `mask`
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
    axes: [i32; 6],
}

#[derive(PartialEq, Eq)]
pub enum StarboardInput {
    Axis { id: u32, value: i32 },
    Button { id: u32, value: bool },
}
