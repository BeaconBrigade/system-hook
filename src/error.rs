use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
#[error("could not parse address into ip address or system path")]
pub struct TcpOrUnixParseError;
