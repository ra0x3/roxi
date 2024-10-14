use crate::{
    auth::SharedKeyAuthentication, config::Config as ServerConfig, error::ServerError,
    ServerResult,
};
use async_std::sync::{Arc, RwLock};
use roxi_client::Config as ClientConfig;
use roxi_lib::types::ClientId;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::time::{self, Duration};

#[derive(Debug, Hash)]
pub struct Session {
    time: SystemTime,
    expiry: Duration,
    config: ClientConfig,
}

impl Session {
    pub fn new(session_ttl: u64, config: &ClientConfig) -> Self {
        Self {
            time: SystemTime::now(),
            config: config.clone(),
            expiry: Duration::new(session_ttl, 0),
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
    config: ServerConfig,
    auth: SharedKeyAuthentication,
}

impl SessionManager {
    pub fn new(config: ServerConfig) -> Self {
        let key = config.shared_key();
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            auth: SharedKeyAuthentication::new(&key),
        }
    }

    pub async fn authenticate(
        &self,
        client_id: &ClientId,
        client_config: &ClientConfig,
    ) -> ServerResult<()> {
        if let Err(e) = self.auth.authenticate(&client_config.shared_key()) {
            tracing::error!("Failed to authenticate client({client_id}): {e}");
            return Err(ServerError::Unauthenticated);
        }

        self.sessions.write().await.insert(
            client_id.clone(),
            Session::new(self.config.session_ttl(), client_config),
        );
        Ok(())
    }

    pub async fn exists(&self, client_id: &ClientId) -> bool {
        self.sessions.read().await.contains_key(client_id)
    }

    #[allow(unused)]
    pub async fn get_peer_for_gateway(&self) {
        todo!()
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
        let mut interval = time::interval(Duration::from_secs(30));
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
mod tests {}
