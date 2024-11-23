use crate::{
    auth::SharedKeyAuthentication, config::Config as ServerConfig, error::ServerError,
    ServerResult,
};
use async_std::sync::{Arc, RwLock};
use rand::{seq::SliceRandom, thread_rng};
use roxi_client::Config as ClientConfig;
use roxi_lib::types::{Address, ClientId, InterfaceKind};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::time::{self, Duration};

#[derive(Debug, Hash, Clone)]
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

    pub fn gateway_remote_addr(&self) -> ServerResult<Address> {
        let addr =
            Address::try_from(self.config.gateway_remote_addr(InterfaceKind::Tcp))?;
        Ok(addr)
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

        tracing::info!("{client_id:?} authenticated. Adding to sessions");
        self.sessions.write().await.insert(
            client_id.clone(),
            Session::new(self.config.session_ttl(), client_config),
        );
        Ok(())
    }

    pub async fn exists(&self, client_id: &ClientId) -> bool {
        self.sessions.read().await.contains_key(client_id)
    }

    pub async fn get_peer_for_gateway(&self, other: &ClientId) -> ServerResult<Address> {
        let items = self
            .sessions
            .read()
            .await
            .iter()
            .filter_map(|(k, v)| {
                if k != other {
                    return Some(v.clone());
                }
                None
            })
            .collect::<Vec<Session>>();
        tracing::info!("Selecting gateway peer from sessions: {items:?}");
        let mut rng = thread_rng();
        if let Some(session) = items.choose(&mut rng).cloned() {
            return session.gateway_remote_addr();
        }

        Err(ServerError::NoAvailablePeers)
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


    pub async fn clear(&self) -> ServerResult<()> {
      Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_get_peer_for_gateway() {
        let content1 = r#"
path: /Users/rashad/dev/repos/roxi/client.local.yaml

network:
  nat:
    delay: 2
    attempts: 3

  server:
    interface: "0.0.0.0"
    ip: "18.117.198.57"
    ports:
      tcp: 8080
      udp: 5675

  stun:
    ip: ~
    port: ~

  gateway:
    interface: "0.0.0.0"
    ip: "192.168.1.228"
    ports:
      tcp: 8081
      udp: 5677
    max_clients: 10

  wireguard:
    type: "wgquick"
    wgquick:
      config: "/Users/rashad/dev/repos/roxi/wg0.conf.example"
    boringtun:
      private_key: "<ServerPrivateKey>"
      public_key: "<ServerPublicKey>"
      address: "10.0.0.1"
      network_size: "24"
      port: 51820
      dns: "1.1.1.1"
      peers:
        - public_key: "<ServerPublicKey>"
          allowed_ips: "10.0.0.2/32"
          endpoint: "<ClientIPAddress>:51820"
          persistent_keep_alive: 25

auth:
  shared_key: "roxi-XXX"
"#;

        let content2 = r#"
path: /Users/rashad/dev/repos/roxi/client.local.yaml

network:
  nat:
    delay: 2
    attempts: 3

  server:
    interface: "0.0.0.0"
    ip: "18.117.198.57"
    ports:
      tcp: 8080
      udp: 5675

  stun:
    ip: ~
    port: ~

  gateway:
    interface: "0.0.0.0"
    ip: "192.168.1.227"
    ports:
      tcp: 8081
      udp: 5677
    max_clients: 10

  wireguard:
    type: "wgquick"
    wgquick:
      config: "/Users/rashad/dev/repos/roxi/wg0.conf.example"
    boringtun:
      private_key: "<ServerPrivateKey>"
      public_key: "<ServerPublicKey>"
      address: "10.0.0.1"
      network_size: "24"
      port: 51820
      dns: "1.1.1.1"
      peers:
        - public_key: "<ServerPublicKey>"
          allowed_ips: "10.0.0.2/32"
          endpoint: "<ClientIPAddress>:51820"
          persistent_keep_alive: 25

auth:
  shared_key: "roxi-XXX"
"#;

        // Must match config above
        let c1 = ClientId::from("192.168.1.228:8081");
        let c2 = ClientId::from("192.168.1.227:8081");

        let c1_config = ClientConfig::try_from(content1).unwrap();
        let c2_config = ClientConfig::try_from(content2).unwrap();

        let p = std::path::Path::new("./../../server.yaml");

        let srv_config = ServerConfig::try_from(p).unwrap();
        let sessions = SessionManager::new(srv_config);

        let _ = sessions.authenticate(&c1, &c1_config).await;
        assert_eq!(sessions.len().await, 1);
        assert!(sessions.exists(&c1).await);
        assert!(!sessions.exists(&c2).await);

        let result = sessions.get_peer_for_gateway(&c1).await;
        assert!(matches!(result, Err(ServerError::NoAvailablePeers)));

        let _ = sessions.authenticate(&c2, &c2_config).await;
        assert_eq!(sessions.len().await, 2);
        assert!(sessions.exists(&c2).await);

        let result = sessions.get_peer_for_gateway(&c1).await.unwrap();
        let expected = Address::try_from(&c2).unwrap();
        assert_eq!(expected, result);

        let result = sessions.get_peer_for_gateway(&c2).await.unwrap();
        let expected = Address::try_from(&c1).unwrap();
        assert_eq!(expected, result);

        sessions.remove(&c1).await;
        assert_eq!(sessions.len().await, 1);
    }
}
