use std::{net::SocketAddr, str::FromStr, path::PathBuf};

use argh::FromArgs;
use serde::{Deserialize, Serialize};

use crate::error::DaemonActionParseError;

/// shook: a webserver that listens for a webhook on
/// a github repo, that will automatically restart your
/// application service after pulling changes
#[derive(Debug, Clone, FromArgs)]
pub struct AppArgs {
    /// configure logging example: 'shook=info,hyper=debug'
    #[argh(option)]
    pub log_level: Option<String>,
    /// file left to log to (defaults to stdout)
    #[argh(option)]
    pub log_file: Option<PathBuf>,
    /// command to run
    #[argh(subcommand)]
    pub action: Action,
}

#[derive(Debug, Clone, FromArgs)]
#[argh(subcommand)]
pub enum Action {
    Init(Init),
    Serve(Serve),
    Daemon(Daemon),
}

/// generate a shook config and service
#[derive(Debug, Clone, FromArgs)]
#[argh(subcommand, name = "init")]
pub struct Init {}

/// activate the webhook server
#[derive(Debug, Clone, FromArgs)]
#[argh(subcommand, name = "serve")]
pub struct Serve {
    /// address to serve on
    #[argh(option, default = "SocketAddr::new([127, 0, 0, 1].into(), 5002)")]
    pub addr: SocketAddr,
}

/// speak with the shook daemon
#[derive(Debug, Clone, FromArgs)]
#[argh(subcommand, name = "daemon")]
pub struct Daemon {
    /// command for the daemon: start | stop | enable
    #[argh(positional)]
    pub action: DaemonAction,
}

/// command for the daemon
#[derive(Debug, Clone, Copy)]
pub enum DaemonAction {
    /// start service
    Start,
    /// stop service
    Stop,
    /// enable service at startup
    Enable,
}

impl FromStr for DaemonAction {
    type Err = DaemonActionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "start" => Ok(Self::Start),
            "stop" => Ok(Self::Stop),
            "enable" => Ok(Self::Enable),
            _ => Err(DaemonActionParseError),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {}
