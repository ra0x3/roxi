use crate::{ip::IpPoolManager, ServerResult};
use async_std::sync::Arc;
use std::net::IpAddr;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Semaphore,
};

pub async fn authenticate_client(_socket: &TcpStream) -> ServerResult<String> {
    // TODO: Allocate client IDs dynamically
    Ok("client123".to_string())
}

pub async fn tunnel_traffic(_socket: &TcpStream, _ip: IpAddr) -> ServerResult<()> {
    Ok(())
}

pub struct Server {
    listener: TcpListener,
    ip_pool: Arc<IpPoolManager>,
    client_limit: Arc<Semaphore>,
}

impl Server {
    // TODO: Global server config should be used to bootstrap server
    pub async fn new(addr: &str) -> ServerResult<Self> {
        let listener = TcpListener::bind(addr).await?;
        let ip_pool = Arc::new(IpPoolManager::new()?);

        // TODO: client limit should come from global server config
        let client_limit = Arc::new(Semaphore::new(10));

        Ok(Self {
            listener,
            ip_pool,
            client_limit,
        })
    }

    #[allow(unused)]
    pub async fn handle_client(
        socket: TcpStream,
        ip_pool: Arc<IpPoolManager>,
    ) -> ServerResult<()> {
        let client_id = authenticate_client(&socket).await?;
        let client_ip = ip_pool.assign_ip(&client_id).await?;
        tracing::info!("Assigned IP {client_ip} to client {client_id}");

        tunnel_traffic(&socket, client_ip).await?;

        ip_pool.release_ip(&client_id).await?;
        tracing::info!("IP for client {client_id} released");

        Ok(())
    }

    pub async fn run(&self) -> ServerResult<()> {
        loop {
            let (socket, _) = self.listener.accept().await?;
            let permit = self.client_limit.clone().acquire_owned().await?;
            let ip_pool = Arc::clone(&self.ip_pool);

            tokio::spawn(async move {
                if let Err(e) = Server::handle_client(socket, ip_pool).await {
                    tracing::error!("Error handling client: {e}");
                }

                drop(permit);
            });
        }
    }
}
