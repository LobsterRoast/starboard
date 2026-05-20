use crate::input::{
    StarboardAxisStates, StarboardButtonStates, StarboardInput, StarboardInputPacket,
};

const TEST_BUTTON_STATES: StarboardButtonStates = StarboardButtonStates { raw: 13 }; // 0b1101
const TEST_AXIS_STATES: StarboardAxisStates = StarboardAxisStates {
    axes: [0, 145, 223, 1125, 102, 255],
};
