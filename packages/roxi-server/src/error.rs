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
}

impl From<ring::error::Unspecified> for ServerError {
    fn from(_: ring::error::Unspecified) -> Self {
        ServerError::Unspecified
    }
}
