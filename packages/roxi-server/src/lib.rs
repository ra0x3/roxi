use serde::{Deserialize, Serialize};

pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod handler;
pub(crate) mod ip;
pub(crate) mod protocol;
pub(crate) mod server;
pub(crate) mod session;
pub(crate) mod tun;

pub type ServerResult<T> = core::result::Result<T, error::ServerError>;

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct ClientId(String);

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ClientId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

pub use config::Config;
pub use error::ServerError;
pub use ip::IpPoolManager;
pub use protocol::Protocol;
pub use server::Server;
