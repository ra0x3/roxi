pub(crate) mod auth;
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod gateway;
pub(crate) mod handler;
pub(crate) mod ip;
pub(crate) mod protocol;
pub(crate) mod server;
pub(crate) mod session;
pub(crate) mod tun;
pub(crate) mod wireguard;

pub type ServerResult<T> = core::result::Result<T, error::ServerError>;

pub use config::Config;
pub use error::ServerError;
pub use gateway::Gateway;
pub use ip::IpPoolManager;
pub use protocol::Protocol;
pub use server::Server;
