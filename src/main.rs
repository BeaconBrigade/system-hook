mod config;
mod daemon;
mod error;
mod init;
mod server;

use std::fs::OpenOptions;

use config::{Action, AppArgs};
use std::fs::File;
use tracing_subscriber::prelude::*;

#[cfg(not(unix))]
compile_error!("this program only runs on linux");

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _ = dotenvy::dotenv().map_err(|_| {
        tracing::warn!("no '.env' file found");
    });
    let args: AppArgs = argh::from_env();

    let log_writer = match args.log_file {
        Some(path) => OpenOptions::new().create(true).append(true).open(path)?,
        None => File::create("/dev/stdout")?,
    };
    tracing_subscriber::registry()
        .with(args.log_level.map(Into::into).unwrap_or_else(|| {
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shook=INFO".into())
        }))
        .with(tracing_subscriber::fmt::layer().with_writer(log_writer))
        .init();

    match args.action {
        Action::Init(init) => init::init_project(init),
        Action::Serve(serve) => server::serve(serve).await,
        Action::Daemon(daemon) => daemon::daemon_message(daemon),
        Action::Version(_) => {
            println!("shook version: {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }?;

    Ok(())
}
