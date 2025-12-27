use std::{env, io::Write};

use tokio::{
    fs::{DirBuilder, File, OpenOptions},
    io::AsyncWriteExt,
};

fn gen_systemd_unit_file() -> Vec<u8> {
    let mut lines = Vec::new();

    // Add [Unit] header to file
    let unit_args = vec![
        "[Unit]",
        "Description=A daemon to receive and process Starboard inputs",
        "",
    ];

    // Add [Service] header to file
    let service_args = vec![
        "[Service]",
        "Type=simple",
        "User=root",
        "Group=root",
        "ExecStart=/usr/local/bin/starboard server",
        "",
    ];

    for arg in &unit_args {
        let _ = writeln!(&mut lines, "{}", arg);
    }

    for arg in &service_args {
        let _ = writeln!(&mut lines, "{}", arg);
    }

    lines
}

pub async fn create_systemd_unit_file() {
    let _ = DirBuilder::new()
        .recursive(true)
        .create("/etc/systemd/system")
        .await;

    let systemd_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .append(true)
        .open("/etc/systemd/system/starboard.service")
        .await;

    if let Ok(mut systemd_file) = systemd_file {
        let _ = systemd_file.write(gen_systemd_unit_file().as_slice()).await;
        let _ = systemd_file.flush().await;
        println!("Successfully generated starboard.service in /etc/systemd/system");
        return;
    }

    println!("starboard.service already exists. Skipping...")
}
