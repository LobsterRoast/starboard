use bincode::config::Configuration;

static BINCODE_CONFIG: Configuration = bincode::config::standard();

// Return a formatted address (i.e. 255.255.255.255:8080) or a specified default
pub fn format_addr(ip: [u8; 4], port: u16) -> String {
    format!("{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port)
}
