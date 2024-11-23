pub mod utils {

    use roxi_client::{Client, Config as ClientConfig};
    use roxi_lib::constant;
    use roxi_server::{Config as ServerConfig, Server};
    use std::{fs::{File, self}, io::Write, path::Path};
    use std::sync::Arc;

    pub const IP_ONE: &str = "192.168.1.1";
    pub const IP_TWO: &str = "192.168.1.2";
    pub const IP_THREE: &str = "192.168.1.3";
    pub const IP_FOUR: &str = "192.168.1.4";

    pub fn yaml_filename(input: &str) -> String {
        format!("{input}.yaml")
    }

    pub fn wgconf_name(input: &str) -> String {
        format!("{input}.{{constant::WIREGUARD_INTERFACE}}.conf")
    }

    pub async fn bootstrap_new_server() -> Server {
        let (path, content) = server_config_content();

        File::create(&path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let config = ServerConfig::try_from(path.as_str()).unwrap();
        
        Server::new(config).await.unwrap()
    }

    pub async fn bootstrap_new_peer(ip: &str) -> Client {
        let (client_content, client_filepath) = peer_client_config_content(ip);
        let (wg_content, wg_filepath) = peer_wireguard_config_content(ip);

        File::create(&client_filepath)
            .unwrap()
            .write_all(client_content.as_bytes())
            .unwrap();

        File::create(&wg_filepath)
            .unwrap()
            .write_all(wg_content.as_bytes())
            .unwrap();

          let config = ClientConfig::try_from(client_filepath.as_str()).unwrap();

        Client::new(config).await.unwrap()
    }

    pub fn server_config_content() -> (String, String) {
        let path =
            Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(constant::SERVER_FILENAME);
        let path = path.display();

        (
            path.to_string(),
            format!(
                r#"
path: {path}

network:
  server:
    ip: "192.168.1.34"
    interface: "0.0.0.0"
    ports:
      tcp: 8080
      udp: 5675
    max_clients: 10

auth:
  shared_key: "roxi-XXX"
  session_ttl: 3600

          "#
            ),
        )
    }

    pub fn peer_wireguard_config_content(ip: &str) -> (String, String) {
        let path = Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(wgconf_name(ip));
        let path = path.display();
        (
            path.to_string(),
            format!(
                r#"
[Interface]
PrivateKey = $PRIVATE_KEY
Address = {ip}/24
ListenPort = 51820
# PostUp = iptables -A FORWARD -i $INTERFACE -j ACCEPT; iptables -A FORWARD -o $INTERFACE -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
# PostDown = iptables -D FORWARD -i $INTERFACE -j ACCEPT; iptables -D FORWARD -o $INTERFACE -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE
            "#
            ),
        )
    }

    pub fn peer_client_config_content(ip: &str) -> (String, String) {
        let client =
            Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(yaml_filename(ip));
        let wgconf =
            Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(wgconf_name(ip));

        let client = client.display();
        let wgconf = wgconf.display();
        (
          client.to_string(),
            format!(
                r#"
    path: {client}

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

    pub fn bootstrap_env() {}

    pub fn teardown_env() {}

    pub async fn demolish_client(mut client: Client) {

      client.stop().await.unwrap();
        let yml = Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(constant::CLIENT_FILENAME);
        fs::remove_file(yml).unwrap();

        let wgconf = Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(constant::WIREGUARD_INTERFACE);
        fs::remove_file(wgconf).unwrap();
    }

    pub async fn demolish_server(srv: Arc<Server>) {
        srv.clone().stop().await.unwrap();

        let yml = Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(constant::SERVER_FILENAME);
        fs::remove_file(yml).unwrap();

        let wgconf = Path::new(constant::ROXI_CONFIG_DIR_REALPATH).join(constant::WIREGUARD_INTERFACE);
        fs::remove_file(wgconf).unwrap();
    }
}

#[cfg(test)]
mod integration_tests {
    use async_std::sync::Arc;
    use tokio::time::{timeout, Duration};
    use crate::utils::*;

    #[tokio::test]
    async fn test_peer_server_rpc_ping() {
        let srv = bootstrap_new_server().await;
        let mut peer = bootstrap_new_peer(IP_ONE).await;
        let srv = Arc::new(srv);
        let handle = tokio::spawn({
            let srvc = Arc::clone(&srv);
            async move { srvc.run().await }
        });

        let ping = timeout(Duration::from_secs(5), peer.ping()).await;
        assert!(ping.is_ok(), "Ping request timed out or failed");
        assert!(ping.unwrap().is_ok(), "Ping failed on server response");

        handle.abort();
    }
}
