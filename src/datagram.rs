// Return a formatted address (i.e. 255.255.255.255:8080) or a specified default
pub fn format_addr_or(
    ip: Option<[u8; 4]>,
    port: Option<u16>,
    default_ip: [u8; 4],
    default_port: u16,
) -> String {
    let ip = ip.unwrap_or(default_ip);
    let port = port.unwrap_or(default_port);
    format!("{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port)
}
