use std::{env, io::Write};

use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
};

pub fn gen_systemd_unit_file() -> Vec<u8> {
    let mut lines = Vec::new();
    let starboard_path = env::current_exe().unwrap().display().to_string();

    // Add [Unit] header to file
    let unit_args: Vec<&str> = vec![
        "[Unit]",
        "Description=A daemon to receive and process Starboard inputs",
        "",
    ];

    let exec_start = format!("ExecStart={} server", starboard_path);
    // Add [Service] header to file
    let service_args = vec![
        "[Service]",
        "Type=dbus",
        "BusName=starboard",
        &exec_start,
        "",
    ];

    let install_args = vec!["[Install]", "WantedBy=multi-user.target"];

    for arg in &unit_args {
        let _ = writeln!(&mut lines, "{}", arg);
    }

    for arg in &service_args {
        let _ = writeln!(&mut lines, "{}", arg);
    }

    for arg in &install_args {
        let _ = writeln!(&mut lines, "{}", arg);
    }

    lines
}

pub async fn create_systemd_unit_file() {
    let mut systemd_file = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .append(true)
        .open("/usr/lib/systemd/system/starboard.service")
        .await
        .expect("Unable to generate systemd unit file");

    let _ = systemd_file.write(gen_systemd_unit_file().as_slice());
    println!("Successfully generated starboard.service.");
}
