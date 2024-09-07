use crate::{error::ServerError, ServerResult};
use async_std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SystemTime>>>,
    key: String,
}

impl SessionManager {
    pub fn new(key: String) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            key,
        }
    }

    // TODO: Swap all client_id: &str to ClientId
    pub async fn authenticate(&self, client_id: &str, key: &str) -> ServerResult<()> {
        if key == self.key {
            self.sessions
                .write()
                .await
                .insert(client_id.to_string(), SystemTime::now());
            return Ok(());
        }

        Err(ServerError::InvalidSharedKey)
    }

    pub async fn session_exists(&self, client_id: &str) -> bool {
        self.sessions.read().await.contains_key(client_id)
    }

    pub async fn remove_session(&self, client_id: &str) {
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
