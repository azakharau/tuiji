use std::process::{Command, Stdio};

use color_eyre::eyre::eyre;

pub fn open_url(url: &str) -> color_eyre::Result<()> {
    let mut command = if cfg!(target_os = "linux") {
        Command::new("xdg-open")
    } else if cfg!(target_os = "macos") {
        Command::new("open")
    } else if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", ""]);
        command
    } else {
        return Err(eyre!("Opening URLs is not supported on this platform"));
    };

    command
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}
