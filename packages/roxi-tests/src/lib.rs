#[cfg(test)]
mod tests {

    use async_std::sync::Arc;
    use roxi_server::{IpPoolManager, Protocol, Server};
    use tokio::net::TcpStream;

    #[tokio::test]
    pub async fn test_vpn_integration() {
        let addr = "127.0.0.1:8080";
        let server = Server::new(addr).await.unwrap();
        let server_task = tokio::spawn(async move {
            server.run().await.unwrap();
        });

        for i in 1..=3 {
            let client_task = tokio::spawn(async move {
                let stream = TcpStream::connect(addr).await.unwrap();
                let proto = Protocol::new().unwrap();
                let _ = Server::handle_client(
                    stream,
                    Arc::new(IpPoolManager::new().unwrap()),
                )
                .await;
            });

            client_task.await.unwrap();
        }

        server_task.abort();
    }
}
