use std::{net::SocketAddr, path::PathBuf, str::FromStr};

use argh::FromArgs;
use github_webhook_extract::EventDiscriminants;
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
    /// unix user name to run git with
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
    /// the unix group to put the unix socket under. should be group your server/proxy
    /// is running using. e.g.: if nginx: www-data (only applicable if serving over unix socket)
    #[argh(option)]
    pub socket_group: Option<String>,
    /// the owner of the unix socket. should be set to the user your server/proxy
    /// is running under. e.g.: if nginx the user should be www-data
    #[argh(option)]
    pub socket_user: Option<String>,
    /// a command to run before restarting the server service. for example recompiling
    /// an executable.
    #[argh(option)]
    pub pre_restart_command: Option<String>,
    /// name of the .service file for shook
    #[argh(option)]
    pub shook_service_name: Option<String>,
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
    pub socket_group: String,
    pub socket_user: String,
    pub pre_restart_command: String,
    pub shook_service_name: String,
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
    /// override the unix group to put the unix socket under. should be group your server/proxy
    /// is running using. e.g.: if nginx: www-data (only applicable if serving over unix socket)
    #[argh(option)]
    pub socket_group: Option<String>,
    /// override the owner of the unix socket. should be set to the user your server/proxy
    /// is running under. e.g.: if nginx the user should be www-data
    #[argh(option)]
    pub socket_user: Option<String>,
    /// a command to run before restarting the server service. for example recompiling
    /// an executable.
    #[argh(option)]
    pub pre_restart_command: Option<String>,
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
    /// location of shook.toml
    #[argh(positional)]
    pub repo_path: PathBuf,
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
    /// override the unix group to put the unix socket under. should be group your server/proxy
    /// is running using. e.g.: if nginx: www-data (only applicable if serving over unix socket)
    pub socket_group: String,
    /// override the owner of the unix socket. should be set to the user your server/proxy
    /// is running under. e.g.: if nginx the user should be www-data
    pub socket_user: String,
    /// a command to run before restarting the server service. for example recompiling
    /// an executable.
    pub pre_restart_command: String,
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
        if let Some(u) = cli.username {
            self.username = u;
        }
        if let Some(r) = cli.remote {
            self.remote = r;
        }
        if let Some(b) = cli.branch {
            self.branch = b;
        }
        if let Some(g) = cli.socket_group {
            self.socket_group = g;
        }
        if let Some(u) = cli.socket_user {
            self.socket_user = u;
        }
        if let Some(c) = cli.pre_restart_command {
            self.pre_restart_command = c;
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

        Ok(Self::Unix(PathBuf::from_str(s).unwrap()))
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
