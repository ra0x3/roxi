pub mod command;
pub(crate) mod error;
pub(crate) mod message;
pub(crate) mod wireguard;

pub type ProtoResult<T> = core::result::Result<T, error::ProtoError>;

pub use error::ProtoError;
pub use message::{Message, MessageKind, MessageStatus};
pub use wireguard::{
    WireGuardConfig, WireGuardConfigBuilder, WireGuardKey, WireGuardKeyPair,
    WireGuardPeer,
};
