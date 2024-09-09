use crate::{
    auth::SharedKeyAuthentication, config::Config, error::ServerError, ServerResult,
};
use async_std::sync::{Arc, Mutex, RwLock};
use roxi_lib::types::{ClientId, SharedKey};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::time::{self, Duration};

#[derive(Debug, Hash, PartialEq)]
pub struct Session {
    time: SystemTime,
    expiry: Duration,
}

impl Session {
    pub fn new() -> Self {
        Self {
            time: SystemTime::now(),
            expiry: Duration::new(3600, 0),
        }
    }

    pub fn is_idle(&self) -> bool {
        // TODO: Implement this
        false
    }

    pub fn expired(&self) -> bool {
        self.time.elapsed().unwrap_or_default() > self.expiry
    }
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<ClientId, Session>>>,
    config: Config,
    auth: SharedKeyAuthentication,
}

impl SessionManager {
    pub fn new(config: Config) -> Self {
        let key = config.shared_key();
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            auth: SharedKeyAuthentication::new(key),
        }
    }

    pub async fn authenticate(
        &self,
        client_id: &ClientId,
        key: &SharedKey,
    ) -> ServerResult<()> {
        if let Err(e) = self.auth.authenticate(&key) {
            tracing::error!("Failed to authenticate client({client_id}): {e}");
            return Err(ServerError::Unauthenticated);
        }

        self.sessions
            .write()
            .await
            .insert(client_id.clone(), Session::new());
        Ok(())
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
        self.sessions
            .write()
            .await
            .retain(|_, session| !session.expired());
    }

    pub async fn monitor_sessions(&self) {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            tracing::info!("Pruning idle sessions");
            self.sessions
                .write()
                .await
                .retain(|_, session| !session.is_idle());
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_can_interact_with_session_manager() {
        let key = SharedKey("abcd123");
        let client = ClientId::from("client123");
        let manager = SessionManager::new(key.to_string());
        let _ = manager.authenticate(client, key).await;

        assert!(manager.session_exists(client).await);

        manager.remove_session(client).await;
        assert!(!manager.session_exists(client).await);
    }
}
