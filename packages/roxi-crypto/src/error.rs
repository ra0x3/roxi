use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Unspecified ring error")]
    Unspecified,
}

impl From<ring::error::Unspecified> for CryptoError {
    fn from(_: ring::error::Unspecified) -> Self {
        CryptoError::Unspecified
    }
}
