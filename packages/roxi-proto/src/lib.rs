pub(crate) mod error;
pub(crate) mod message;

pub type ProtoResult<T> = core::result::Result<T, error::ProtoError>;

pub use error::ProtoError;
pub use message::{Message, MessageKind, MessageStatus};
