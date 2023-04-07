use std::process::{Command, Stdio};

use color_eyre::eyre::{eyre, Context};

use crate::config::{Daemon, DaemonAction};

/// send command to the shook systemd service
pub fn daemon_message(args: Daemon) -> color_eyre::Result<()> {
    tracing::info!("talking to daemon");
    tracing::debug!("{:#?}", args);

    match args.action {
        DaemonAction::Start(_) => {
            tracing::info!("starting daemon");

            run_systemctl_command("start")?;
        }
        DaemonAction::Stop(_) => {
            tracing::info!("stopping daemon");

            run_systemctl_command("stop")?;
        }
        DaemonAction::Enable(_) => {
            tracing::info!("enabling daemon");

            run_systemctl_command("enable")?;
        }
    };

    tracing::info!("completed message");

    Ok(())
}

/// run a systemctl command, pipe the output and return if it doesn't exit with 0
fn run_systemctl_command(name: &str) -> color_eyre::Result<()> {
    let mut handle = Command::new("systemctl")
        .arg(name)
        .arg("shook.service")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawning systemctl")?;

    let status = handle.wait()?;

    tracing::info!(
        "systemctl returned status: {}",
        status
            .code()
            .map(|s| s.to_string())
            .unwrap_or("<terminated by signal>".to_string())
    );

    if status.code().unwrap_or(1) != 0 {
        return Err(eyre!("systemctl returned error"));
    }

    Ok(())
}
