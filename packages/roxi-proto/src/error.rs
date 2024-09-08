use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtoError {
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Malformed message")]
    MalformedMessage,

    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
