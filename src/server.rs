pub struct StarboardServerBuilder {
    // A struct to help build a server
    ip: String,
    port: u16,
    enabled_buttons: u32,
    enabled_axes: u32,
}

pub struct StarboardServer {
    // The server is what will receive input packets from the controller and simulate a virtual
    // joystick on another PC
    address: String,     // i.e. {ip}:{port}
    enabled_inputs: u32, // Bitmask representing all the enabled buttons on the server
    enabled_axes: u32,   // Bitmask representing all the enabled axes on the
                         // server
}
