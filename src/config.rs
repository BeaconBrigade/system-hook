use std::{net::SocketAddr, path::PathBuf, str::FromStr};

use argh::FromArgs;
use github_webhook::EventDiscriminants;
use serde::{Deserialize, Serialize};

use crate::error::TcpOrUnixParseError;

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
pub struct Init {
    /// linux user name to run git with
    #[argh(option)]
    pub username: Option<String>,
    /// path to the repository
    #[argh(option)]
    pub repo_path: Option<PathBuf>,
    /// the remote to track for pulling changes
    #[argh(option)]
    pub remote: Option<String>,
    /// the branch to track for pulling changes
    #[argh(option)]
    pub branch: Option<String>,
    /// name of systemd service to update when receiving a github event
    #[argh(option)]
    pub system_name: Option<String>,
    /// allowed github events to update the server after receiving
    #[argh(option, from_str_fn(parse_multiple_events))]
    pub update_events: Option<Vec<EventDiscriminants>>,
    /// address to serve on: a path to a unix socket, or an ip address for tcp
    #[argh(option)]
    pub addr: Option<TcpOrUnix>,
}

/// init args without all the options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitConfig {
    pub username: String,
    pub repo_path: PathBuf,
    pub remote: String,
    pub branch: String,
    pub system_name: String,
    pub update_events: Vec<EventDiscriminants>,
    pub addr: TcpOrUnix,
}

/// activate the webhook server - each argument overrides the value in
/// your `shook.toml`
#[derive(Debug, Clone, FromArgs)]
#[argh(subcommand, name = "serve")]
pub struct Serve {
    /// linux user name to run git with
    #[argh(option)]
    pub username: Option<String>,
    /// override path to the repository
    #[argh(option)]
    pub repo_path: Option<PathBuf>,
    /// override remote to track for pulling changes
    #[argh(option)]
    pub remote: Option<String>,
    /// override branch to track for pulling changes
    #[argh(option)]
    pub branch: Option<String>,
    /// override name of systemd service to update when receiving a github event
    #[argh(option)]
    pub system_name: Option<String>,
    /// override github events to update the server after receiving
    #[argh(option, from_str_fn(parse_multiple_events))]
    pub update_events: Option<Vec<EventDiscriminants>>,
    /// override address to serve on: a path to a unix socket, or an ip address for tcp
    #[argh(option)]
    pub addr: Option<TcpOrUnix>,
}

/// parse a string like: 'commit,push' into events to listen to
pub fn parse_multiple_events(s: &str) -> Result<Vec<EventDiscriminants>, String> {
    s.split(',')
        .map(|s| EventDiscriminants::from_str(s).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()
}

/// speak with the shook daemon
#[derive(Debug, Clone, FromArgs)]
#[argh(subcommand, name = "daemon")]
pub struct Daemon {
    /// command for the daemon
    #[argh(subcommand)]
    pub action: DaemonAction,
}

/// command for the daemon
#[derive(Debug, Clone, Copy, FromArgs)]
#[argh(subcommand)]
pub enum DaemonAction {
    Start(DaemonStart),
    Enable(DaemonEnable),
    Stop(DaemonStop),
}

/// start service
#[derive(Debug, Clone, Copy, FromArgs)]
#[argh(subcommand, name = "start")]
pub struct DaemonStart {}

/// stop service
#[derive(Debug, Clone, Copy, FromArgs)]
#[argh(subcommand, name = "stop")]
pub struct DaemonStop {}

/// enable service at startup
#[derive(Debug, Clone, Copy, FromArgs)]
#[argh(subcommand, name = "enable")]
pub struct DaemonEnable {}

/// server configuration parsed from `shook.toml`
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// linux user name to run git with
    pub username: String,
    /// path to the repository
    pub repo_path: PathBuf,
    /// the remote to track for pulling changes
    pub remote: String,
    /// the branch to track for pulling changes
    pub branch: String,
    /// name of systemd service to update when receiving a github event
    pub system_name: String,
    /// github events to update the server after receiving
    pub update_events: Vec<EventDiscriminants>,
    /// address to serve on: a path to a unix socket, or an ip address for tcp
    pub addr: TcpOrUnix,
}

impl ServerConfig {
    pub fn merge(&mut self, cli: Serve) {
        if let Some(p) = cli.repo_path {
            self.repo_path = p;
        }
        if let Some(n) = cli.system_name {
            self.system_name = n;
        }
        if let Some(e) = cli.update_events {
            self.update_events = e;
        }
        if let Some(a) = cli.addr {
            self.addr = a;
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum TcpOrUnix {
    Tcp(SocketAddr),
    Unix(PathBuf),
}

impl FromStr for TcpOrUnix {
    type Err = TcpOrUnixParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(s) = SocketAddr::from_str(s) {
            return Ok(Self::Tcp(s));
        }

        if let Ok(p) = PathBuf::from_str(s) {
            return Ok(Self::Unix(p));
        }

        Err(TcpOrUnixParseError)
    }
}

impl ToString for TcpOrUnix {
    fn to_string(&self) -> String {
        match self {
            Self::Tcp(s) => format!("{}", s),
            Self::Unix(p) => format!("{}", p.to_string_lossy()),
        }
    }
}
