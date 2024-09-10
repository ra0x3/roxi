use crate::{
    auth::SharedKeyAuthentication, config::Config, error::ServerError, ServerResult,
};
use async_std::sync::{Arc, RwLock};
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
    pub fn new(ttl: u64) -> Self {
        Self {
            time: SystemTime::now(),
            expiry: Duration::new(ttl, 0),
        }
    }

    pub fn is_idle(&self) -> bool {
        // TODO: Implement this
        false
    }

    #[allow(unused)]
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
        if let Err(e) = self.auth.authenticate(key) {
            tracing::error!("Failed to authenticate client({client_id}): {e}");
            return Err(ServerError::Unauthenticated);
        }

        self.sessions.write().await.insert(
            client_id.clone(),
            Session::new(self.config.session_ttl() as u64),
        );
        Ok(())
    }

    pub async fn exists(&self, client_id: &ClientId) -> bool {
        self.sessions.read().await.contains_key(client_id)
    }

    #[allow(unused)]
    pub async fn remove(&self, client_id: &ClientId) {
        self.sessions.write().await.remove(client_id);
    }

    #[allow(unused)]
    pub async fn len(&self) -> usize {
        self.sessions.read().await.len()
    }

    #[allow(unused)]
    pub async fn cleanup(&self) {
        self.sessions
            .write()
            .await
            .retain(|_, session| !session.expired());
    }

    pub async fn monitor(&self) {
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

    const CONFIG: &str = r#"
server:
  ip: "192.168.1.34"
  interface: "0.0.0.0"
  port: 8080
  max_clients: 10

tun:
  address: "10.0.0.1"
  destination: "10.0.0.2"
  netmask: "255.255.255.0"
  name: "utun6"

auth:
  shared_key: "roxi-XXX"
  session_ttl: 3600
"#;

    #[tokio::test]
    async fn test_can_interact_with_session_manager() {
        let config = Config::try_from(CONFIG).unwrap();
        let key = SharedKey::from("roxi-XXX");
        let client = ClientId::from("client123");
        let manager = SessionManager::new(config);
        let _ = manager.authenticate(&client, &key).await;

        assert!(manager.exists(&client).await);

        manager.remove(&client).await;
        assert!(!manager.exists(&client).await);
    }
}
