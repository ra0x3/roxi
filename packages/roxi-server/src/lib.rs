pub(crate) mod error;
pub(crate) mod handler;
pub(crate) mod ip;
pub(crate) mod server;

pub type ServerResult<T> = core::result::Result<T, error::ServerError>;
