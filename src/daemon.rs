use crate::config::{Daemon, DaemonAction};

pub fn daemon_message(args: Daemon) -> color_eyre::Result<()> {
    tracing::info!("talking to daemon");
    tracing::debug!("{:#?}", args);

    match args.action {
        DaemonAction::Start => tracing::info!("starting daemon"),
        DaemonAction::Stop => tracing::info!("stopping daemon"),
        DaemonAction::Enable => tracing::info!("enabling daemon"),
    };

    Ok(())
}
