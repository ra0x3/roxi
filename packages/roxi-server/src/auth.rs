use crate::{ServerError, ServerResult};
use roxi_lib::types::SharedKey;

pub struct SharedKeyAuthentication {
    shared_key: SharedKey,
}

impl SharedKeyAuthentication {
    pub fn new(shared_key: SharedKey) -> Self {
        Self { shared_key }
    }
    pub fn authenticate(&self, k: &SharedKey) -> ServerResult<()> {
        if k == &self.shared_key {
            return Ok(());
        }

        Err(ServerError::Unauthenticated)
    }
}
