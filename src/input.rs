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
