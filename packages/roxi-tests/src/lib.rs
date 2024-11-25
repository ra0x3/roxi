pub mod utils {

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

    pub fn wgconf_name(input: &str) -> String {
        format!("{}-{}.conf", input, constant::WIREGUARD_INTERFACE)
    }

    pub async fn setup_server(ip: &str) -> Server {
        let (path, content) = server_config_content(ip);

        File::create(&path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let path = Path::new(&path);
        let config = ServerConfig::try_from(path).unwrap();

        Server::new(config).await.unwrap()
    }

    pub async fn setup_peer(ip: &str) -> Client {
        let (peer_filepath, peer_content) = peer_config_content(ip);
        let (wg_filepath, wg_content) = peer_wireguard_config_content(ip);

        File::create(&peer_filepath)
            .unwrap()
            .write_all(peer_content.as_bytes())
            .unwrap();

        File::create(&wg_filepath)
            .unwrap()
            .write_all(wg_content.as_bytes())
            .unwrap();

        let p = Path::new(&peer_filepath);
        let config = ClientConfig::try_from(p).unwrap();

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
    ip: "192.168.1.34"
    interface: "0.0.0.0"
    ports:
      tcp: 8080
      udp: 5675
    max_clients: 10
    response_timeout: 2

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
        let path = Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(wgconf_name(ip));
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
        let wgconf = Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(wgconf_name(ip));

        let client = expand_tilde(&client).display().to_string();
        let wgconf = expand_tilde(&wgconf).display().to_string();

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
      udp: 5675

  stun:
    ip: ~
    port: ~

  gateway:
    interface: "0.0.0.0"
    ip: "127.0.0.1"
    ports:
      tcp: 8081
      udp: 5677
    max_clients: 10

  wireguard:
    type: "wgquick"
    wgquick:
      config: "{wgconf}"
    boringtun:
      private_key: "<ServerPrivateKey>"
      public_key: "<ServerPublicKey>"
      address: "10.0.0.1"
      network_size: "24"
      port: 51820
      dns: "1.1.1.1"
      peers:
        - public_key: "<ClientPublicKey>"
          allowed_ips: "10.0.0.2/32"
          endpoint: "<ClientIPAddress>:51820"
          persistent_keep_alive: 25

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
    use tokio::time::{timeout, Duration};

    static INIT: std::sync::Once = std::sync::Once::new();

    fn init_logging() {
        INIT.call_once(|| {
            tracing_subscriber::fmt().with_test_writer().init();
        });
    }

    const TIMEOUT_SECS: u64 = 0x1;

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

        let ping = timeout(Duration::from_secs(TIMEOUT_SECS), peer.ping()).await;
        assert!(ping.is_ok(), "Ping request timed out or failed");
        assert!(ping.unwrap().is_ok(), "Ping failed on server response");

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

        let response =
            timeout(Duration::from_secs(TIMEOUT_SECS), peer.authenticate()).await;
        assert!(response.is_ok(), "Authenticate request timed out or failed");
        assert!(
            response.unwrap().is_ok(),
            "Authenticate failed on server response"
        );

        handle.abort();

        peer.stop().await.unwrap();
        srv.clone().stop().await.unwrap();
        cleanup_config_files().await;
    }
}
