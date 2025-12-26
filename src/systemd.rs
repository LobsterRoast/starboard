use std::{env, io::Write};

pub fn gen_systemd_unit_file() -> String {
    let mut lines = Vec::new();
    let starboard_path = env::current_exe().unwrap().display().to_string();

    // Add [Unit] header to file
    let unit_args: Vec<&str> = vec![
        "[Unit]",
        "Description=A daemon to receive and process Starboard inputs",
    ];

    let exec_start = format!("ExecStart={} server", starboard_path);
    // Add [Service] header to file
    let service_args = vec!["[Service]", "Type=dbus", "BusName=starboard", &exec_start];
    for arg in &unit_args {
        let _ = writeln!(&mut lines, "{}", arg);
    }

    for arg in &service_args {
        let _ = writeln!(&mut lines, "{}", arg);
    }

    String::from_utf8(lines).unwrap()
}
