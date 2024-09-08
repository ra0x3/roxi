pub(crate) mod client;
pub(crate) mod config;
pub(crate) mod error;

pub type ClientResult<T> = core::result::Result<T, error::ClientError>;

pub use client::Client;
pub use config::Config;
pub use error::ClientError;
