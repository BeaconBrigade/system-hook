use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
#[error("could not parse daemon command, options: start | stop | enable")]
pub struct DaemonActionParseError;
