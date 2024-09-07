#[cfg(test)]
mod tests {

    use async_std::sync::Arc;
    use roxi_server::{Config, IpPoolManager, Protocol, Server};
    use tokio::net::TcpStream;

    #[tokio::test]
    pub async fn test_vpn_integration() {
        let content = r#"
host: "127.0.0.1"
port: "8080"

client:
  limit: 10

ip:
  pool:
    - "192.168.1.1"
    - "192.168.1.2"
    - "192.168.1.3"

tun:
  address: "10.0.0.1"
  destination: "10.0.0.2"
  netmask: "255.255.255.0"
  name: "utun6"

session:
  expiry: 3600

auth:
  shared_key: "roxi-XXX"
"#;
        let config = Config::try_from(content).unwrap();
        let server = Server::new(config.clone()).await.unwrap();
        let server_task = tokio::spawn(async move {
            server.run().await.unwrap();
        });

        for i in 1..=3 {
            let addr = config.hostname();
            let config = config.clone();
            let client_task = tokio::spawn(async move {
                let stream = TcpStream::connect(&addr).await.unwrap();
                let proto = Protocol::new().unwrap();
                let _ = Server::handle_client(
                    stream,
                    Arc::new(IpPoolManager::new(config).unwrap()),
                )
                .await;
            });

            client_task.await.unwrap();
        }

        server_task.abort();
    }
}
