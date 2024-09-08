use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("AddrParseError: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("AquireError: {0}")]
    Acquire(#[from] tokio::sync::AcquireError),

    #[error("No IP addresses available")]
    NoIpAddrAvailable,

    #[error("Unspecified ring error")]
    Unspecified,

    #[error("Invalid shared key")]
    InvalidSharedKey,

    #[error("Tun error: {0}")]
    Tun(#[from] tun::Error),

    #[error("Tokio task join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),

    #[error("Serde yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Proto error: {0}")]
    Proto(#[from] roxi_proto::ProtoError),

    #[error("Invalid message")]
    InvalidMessage,

    #[error("Unauthenticated")]
    Unauthenticated,

    #[error("FromUt8 error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
}

impl From<ring::error::Unspecified> for ServerError {
    fn from(_: ring::error::Unspecified) -> Self {
        ServerError::Unspecified
    }
}
