struct StarboardButtonStates {
    // There are 25 different buttons in SDL3, requiring at least a u32 to cover them all.
    raw: u32,
}

impl StarboardButtonStates {
    pub fn delta(&self, other: &StarboardButtonStates) -> u32 {
        // delta() is symmetric.
        // That is, A.delta(&B) == B.delta(&A).

        self.raw ^ other.raw
    }
}

struct StarboardInputPacket {
    buttons: StarboardButtonStates,
    axes: [i32; 6],
}

pub enum StarboardInput {
    Axis { id: u32, value: i32 },
    Button { id: u32, value: bool },
}
