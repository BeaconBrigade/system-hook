use std::{fs::File, io::Read, process::Command};

use color_eyre::eyre::{eyre, Context};

use crate::config::{Daemon, DaemonAction, InitConfig};

/// send command to the shook systemd service
pub fn daemon_message(args: Daemon) -> color_eyre::Result<()> {
    tracing::info!("talking to daemon");
    tracing::debug!("{:#?}", args);

    let mut file = File::open(&args.config_path).context("opening shook config")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .context("reading shook config")?;
    let config: InitConfig = toml::from_str(&buf).context("parsing shook config")?;
    let service_name = config.shook_service_name;

    match args.action {
        DaemonAction::Start(_) => {
            tracing::info!("starting daemon");

            run_systemctl_command("start", &service_name)?;
        }
        DaemonAction::Stop(_) => {
            tracing::info!("stopping daemon");

            run_systemctl_command("stop", &service_name)?;
        }
        DaemonAction::Enable(_) => {
            tracing::info!("enabling daemon");

            run_systemctl_command("enable", &service_name)?;
        }
    };

    tracing::info!("completed message");

    Ok(())
}

/// run a systemctl command, pipe the output and return if it doesn't exit with 0
fn run_systemctl_command(name: &str, service_name: &str) -> color_eyre::Result<()> {
    let mut handle = Command::new("systemctl")
        .arg(name)
        .arg(service_name)
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
