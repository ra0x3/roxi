use crate::{error::ServerError, ServerResult};
use async_std::sync::{Arc, Mutex, RwLock};
use roxi_lib::types::{ClientId, SharedKey};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<ClientId, SystemTime>>>,
    key: SharedKey,
}

impl SessionManager {
    pub fn new(key: SharedKey) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            key,
        }
    }

    pub async fn authenticate(
        &self,
        client_id: &ClientId,
        key: &str,
    ) -> ServerResult<()> {
        let key = SharedKey::from(key);
        if key == self.key {
            self.sessions
                .write()
                .await
                .insert(client_id.clone(), SystemTime::now());
            return Ok(());
        }

        Err(ServerError::InvalidSharedKey)
    }

    pub async fn session_exists(&self, client_id: &ClientId) -> bool {
        self.sessions.read().await.contains_key(client_id)
    }

    pub async fn remove_session(&self, client_id: &ClientId) {
        self.sessions.write().await.remove(client_id);
    }

    pub async fn len(&self) -> usize {
        self.sessions.read().await.len()
    }

    pub async fn drop_expired_sessions(&self) {
        // TODO: Session TTL should come from global server config
        let expiry = Duration::new(3600, 0);
        self.sessions
            .write()
            .await
            .retain(|_, start| start.elapsed().unwrap_or_default() < expiry);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_can_interact_with_session_manager() {
        let key = "abcdefg12345";
        let client = "client1234";
        let manager = SessionManager::new(key.to_string());
        let _ = manager.authenticate(client, key).await;

        assert!(manager.session_exists(client).await);

        manager.remove_session(client).await;
        assert!(!manager.session_exists(client).await);
    }
}
