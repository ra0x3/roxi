use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("AddrParseError: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid shared key")]
    InvalidSharedKey,

    #[error("Serde yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}
