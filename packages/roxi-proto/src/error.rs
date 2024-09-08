use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtoError {
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Malformed message")]
    MalformedMessage,
}
