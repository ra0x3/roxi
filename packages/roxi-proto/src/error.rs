use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtoError {
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Malformed message")]
    MalformedMessage,

    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Toml de error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Crypto error: {0}")]
    Crypto(#[from] roxi_crypto::CryptoError),

    #[error("Unspecified ring error")]
    Unspecified,

    #[error("Toml ser error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Malformed config")]
    MalformedConfig,
}

impl From<ring::error::Unspecified> for ProtoError {
    fn from(_: ring::error::Unspecified) -> Self {
        ProtoError::Unspecified
    }
}
