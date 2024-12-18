pub mod utils {

    use rand::Rng;
    use regex::Regex;
    use roxi_client::{Client, Config as ClientConfig};
    use roxi_lib::constant;
    use roxi_server::{Config as ServerConfig, Server};
    use std::{
        env,
        fs::{self, File},
        io::Write,
        path::{Path, PathBuf},
    };

    pub const IP_ONE: &str = "192.168.1.1";
    pub const IP_TWO: &str = "192.168.1.2";
    pub const IP_THREE: &str = "192.168.1.3";
    pub const IP_FOUR: &str = "192.168.1.4";

    pub fn yaml_filename(input: &str) -> String {
        format!("{input}.yaml")
    }

    pub fn wireguard_config_name(input: &str) -> String {
        format!("{}-{}.conf", input, constant::WIREGUARD_INTERFACE)
    }

    pub async fn setup_server(ip: &str) -> Server {
        let (file, content) = server_config_content(ip);

        File::create(&file)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let config = ServerConfig::try_from(Path::new(&file)).unwrap();

        Server::new(config).await.unwrap()
    }

    pub async fn setup_peer(ip: &str) -> Client {
        let (peer_file, peer_content) = peer_config_content(ip);
        let (wireguard_file, wireguard_content) = peer_wireguard_config_content(ip);

        File::create(&peer_file)
            .unwrap()
            .write_all(peer_content.as_bytes())
            .unwrap();

        File::create(&wireguard_file)
            .unwrap()
            .write_all(wireguard_content.as_bytes())
            .unwrap();

        let config = ClientConfig::try_from(Path::new(&peer_file)).unwrap();
        Client::new(config).await.unwrap()
    }

    pub fn server_config_content(ip: &str) -> (String, String) {
        let name = format!("{}-{}", ip, constant::SERVER_FILENAME);
        let path =
            Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(yaml_filename(&name));
        let path = expand_tilde(&path).display().to_string();

        (
            path.to_string(),
            format!(
                r#"path: {path}

network:
  server:
    ip: "{ip}"
    interface: "0.0.0.0"
    ports:
      tcp: 8080
      udp: 5675
    max_clients: 10
    response_timeout: 1

auth:
  shared_key: "roxi-XXX"
  session_ttl: 3600
"#
            ),
        )
    }

    fn expand_tilde(p: &Path) -> PathBuf {
        if let Some(pstr) = p.to_str() {
            if pstr.starts_with("~") {
                if let Ok(home) = env::var("HOME") {
                    return PathBuf::from(pstr.replacen("~", &home, 1));
                }
            }
        }
        p.to_path_buf()
    }

    pub fn peer_wireguard_config_content(ip: &str) -> (String, String) {
        let path =
            Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(wireguard_config_name(ip));
        let path = expand_tilde(&path).display().to_string();

        (
            path,
            format!(
                r#"[Interface]
PrivateKey = "<ServerPrivateKey>"
Address = "{ip}/24"
ListenPort = 51820
"#
            ),
        )
    }

    pub fn peer_config_content(ip: &str) -> (String, String) {
        let client =
            Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(yaml_filename(ip));
        let wgconf =
            Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(wireguard_config_name(ip));

        let client = expand_tilde(&client).display().to_string();
        let wgconf = expand_tilde(&wgconf).display().to_string();

        let udp_gen = || rand::thread_rng().gen_range(5675..=5685);
        let udp = udp_gen();
        let gateway_udp = udp_gen();

        (
            client.clone(),
            format!(
                r#"path: {client}

network:
  nat:
    delay: 2
    attempts: 3

  server:
    interface: "0.0.0.0"
    ip: "127.0.0.1"
    ports:
      tcp: 8080
      udp: {udp}
    request_timeout: 1
    response_timeout: 1

  stun:
    ip: ~
    port: ~

  gateway:
    interface: "0.0.0.0"
    ip: "{ip}"
    ports:
      tcp: 8081
      udp: {gateway_udp}
    max_clients: 10

  wireguard:
    config: "{wgconf}"

auth:
  shared_key: "roxi-XXX"
  "#
            ),
        )
    }

    pub async fn cleanup_config_files() {
        let rgx = Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\.").unwrap();
        let dir = Path::new(constant::ROXI_CONFIG_DIR_REALPATH);
        let dir = expand_tilde(dir);
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if rgx.is_match(&name) {
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(&path).unwrap();
                }
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::utils::*;
    use async_std::sync::Arc;
    use roxi_client::Config as ClientConfig;
    use roxi_lib::types::Address;
    use roxi_proto::{MessageKind, MessageStatus};
    use roxi_server::{ServerError, SessionManager};
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
        sync::Once,
    };

    static INIT: Once = Once::new();

    fn init_logging() {
        INIT.call_once(|| {
            tracing_subscriber::fmt().with_test_writer().init();
        });

        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(".config").join("roxi");
            if let Err(e) = fs::create_dir_all(&config_path) {
                tracing::error!(
                    "Failed to create config directory at {:?}: {}",
                    config_path,
                    e
                );
            } else {
                tracing::info!("Config directory created at {:?}", config_path);
            }
        } else {
            tracing::error!("Could not determine the home directory");
        }
    }

    mod server {
        use super::*;

        #[tokio::test]
        async fn test_server_sessions_manager() {
            init_logging();

            let srv = setup_server(IP_ONE).await;
            let sessions = SessionManager::new(srv.config().clone());

            let c1 = setup_peer(IP_TWO).await;
            let c2 = setup_peer(IP_THREE).await;

            let _ = sessions.authenticate(&c1.client_id(), c1.config()).await;
            assert_eq!(sessions.len().await, 1);
            assert!(sessions.exists(&c1.client_id()).await);
            assert!(!sessions.exists(&c2.client_id()).await);

            let result = sessions.get_peer_for_gateway(&c1.client_id()).await;
            assert!(matches!(result, Err(ServerError::NoAvailablePeers)));

            let _ = sessions.authenticate(&c2.client_id(), c2.config()).await;
            assert_eq!(sessions.len().await, 2);
            assert!(sessions.exists(&c2.client_id()).await);

            let result = sessions
                .get_peer_for_gateway(&c1.client_id())
                .await
                .unwrap();
            let expected = Address::try_from(&c2.client_id()).unwrap();
            assert_eq!(expected, result);

            let result = sessions
                .get_peer_for_gateway(&c2.client_id())
                .await
                .unwrap();
            let expected = Address::try_from(&c1.client_id()).unwrap();
            assert_eq!(expected, result);

            sessions.remove(&c1.client_id()).await;
            assert_eq!(sessions.len().await, 1);
        }
    }

    mod config {
        use super::*;

        #[tokio::test]
        async fn test_updates_to_wireguard_config_persist() {
            let (file, content) = peer_config_content(IP_ONE);

            File::create(&file)
                .unwrap()
                .write_all(content.as_bytes())
                .unwrap();

            let _config = ClientConfig::try_from(Path::new(&file)).unwrap();
        }
    }

    mod peer_server_interaction {
        use super::*;

        #[tokio::test]
        async fn test_peer_server_rpc_ping() {
            init_logging();
            let srv = setup_server(IP_ONE).await;
            let mut peer = setup_peer(IP_TWO).await;
            let srv = Arc::new(srv);
            let handle = tokio::spawn({
                let srvc = Arc::clone(&srv);
                async move { srvc.run().await }
            });

            let ping = peer.ping().await;
            assert!(ping.is_ok(), "Ping failed or timed out.");
            assert!(ping.as_ref().unwrap().is_some(), "Ping response failed.");

            let ping = ping.unwrap().unwrap();
            assert_eq!(*ping.status(), MessageStatus::r#Ok);
            assert_eq!(*ping.kind(), MessageKind::Pong);

            handle.abort();

            peer.stop().await.unwrap();
            srv.clone().stop().await.unwrap();
            cleanup_config_files().await;
        }

        #[tokio::test]
        async fn test_peer_server_rpc_authenticate() {
            init_logging();
            let srv = setup_server(IP_ONE).await;
            let mut peer = setup_peer(IP_TWO).await;
            let srv = Arc::new(srv);
            let handle = tokio::spawn({
                let srvc = Arc::clone(&srv);
                async move { srvc.run().await }
            });

            let auth = peer.authenticate().await;
            assert!(auth.is_ok(), "Auth failed or timed out.");
            assert!(auth.as_ref().unwrap().is_some(), "Auth response failed.");

            let auth = auth.unwrap().unwrap();
            assert_eq!(*auth.status(), MessageStatus::r#Ok);
            assert_eq!(*auth.kind(), MessageKind::AuthenticationResponse);

            handle.abort();

            peer.stop().await.unwrap();
            srv.clone().stop().await.unwrap();
            cleanup_config_files().await;
        }

        #[tokio::test]
        async fn test_peer_server_rpc_stun() {
            init_logging();
            let srv = setup_server(IP_ONE).await;
            let mut peer = setup_peer(IP_TWO).await;
            let srv = Arc::new(srv);
            let handle = tokio::spawn({
                let srvc = Arc::clone(&srv);
                async move { tokio::join!(srvc.clone().run(), srvc.run_udp()) }
            });

            let stun = peer.stun().await;
            assert!(stun.is_ok(), "STUN failed or timed out.");

            let auth = peer.authenticate().await;
            assert!(auth.is_ok(), "Auth failed or timed out.");
            assert!(auth.as_ref().unwrap().is_some(), "Auth response failed.");
            let auth = auth.unwrap().unwrap();
            assert_eq!(*auth.status(), MessageStatus::r#Ok);
            assert_eq!(*auth.kind(), MessageKind::AuthenticationResponse);
            let stun = peer.stun().await;
            assert!(stun.is_ok(), "STUN failed or timed out.");
            handle.abort();

            peer.stop().await.unwrap();
            srv.clone().stop().await.unwrap();
            cleanup_config_files().await;
        }

        #[tokio::test]
        async fn test_peer_server_rpc_seed() {
            init_logging();
            let srv = setup_server(IP_ONE).await;
            let mut peer = setup_peer(IP_TWO).await;
            let srv = Arc::new(srv);
            let handle = tokio::spawn({
                let srvc = Arc::clone(&srv);
                async move { srvc.run().await }
            });

            let seed = peer.seed().await;
            assert!(seed.is_ok(), "Seed failed or timed out.");
            assert!(seed.as_ref().unwrap().is_some(), "Seed response failed.");

            let auth = peer.authenticate().await;
            assert!(auth.is_ok(), "Auth failed or timed out.");
            assert!(auth.as_ref().unwrap().is_some(), "Auth response failed.");
            let auth = auth.unwrap().unwrap();
            assert_eq!(*auth.status(), MessageStatus::r#Ok);
            assert_eq!(*auth.kind(), MessageKind::AuthenticationResponse);
            let seed = peer.seed().await;
            assert!(seed.is_ok(), "Seed failed or timed out.");
            assert!(seed.as_ref().unwrap().is_some(), "Seed response failed.");

            let seed = seed.unwrap().unwrap();
            assert_eq!(*seed.status(), MessageStatus::r#Ok);
            assert_eq!(*seed.kind(), MessageKind::SeedResponse);

            handle.abort();

            peer.stop().await.unwrap();
            srv.clone().stop().await.unwrap();
            cleanup_config_files().await;
        }
    }

    mod peer_peer_interaction {}
}
