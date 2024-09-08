use crate::{config::Config, error::ServerError, ip::IpPoolManager, ServerResult};
use async_std::sync::Arc;
use roxi_lib::types::ClientId;
use roxi_proto::{Message, MessageKind};
use std::net::Ipv4Addr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Semaphore,
};

pub async fn authenticate_client(_stream: &TcpStream) -> ServerResult<ClientId> {
    // TODO: Allocate client IDs dynamically
    Ok(ClientId::from("client123"))
}

pub async fn tunnel_traffic(_stream: &TcpStream, _ip: Ipv4Addr) -> ServerResult<()> {
    Ok(())
}

pub struct Server {
    listener: TcpListener,
    ip_pool: Arc<IpPoolManager>,
    client_limit: Arc<Semaphore>,
    config: Config,
}

impl Server {
    pub async fn new(config: Config) -> ServerResult<Self> {
        let listener = TcpListener::bind(config.hostname()).await?;
        let ip_pool = Arc::new(IpPoolManager::new(config.clone())?);
        let client_limit = Arc::new(Semaphore::new(config.client_limit()));

        Ok(Self {
            listener,
            ip_pool,
            client_limit,
            config,
        })
    }

    pub async fn handle_client(
        mut stream: TcpStream,
        ip_pool: Arc<IpPoolManager>,
        hostname: String,
    ) -> ServerResult<()> {
        tracing::info!("Handling incoming tcp stream");

        let mut buff = vec![0u8; 1024];
        let n = stream.read(&mut buff).await?;

        if n == 0 {
            return Err(ServerError::ConnectionClosed);
        }

        tracing::info!("Read {n} bytes");

        let msg = Message::deserialize(&buff)?;

        tracing::info!("Received message: {msg:?}");

        match msg.kind() {
            MessageKind::Ping => {
                let msg = Message::new(MessageKind::Pong, hostname, None);
                let data = msg.serialize()?;
                stream.write_all(&data).await?;
            }
            _ => {
                return Err(ServerError::InvalidMessage);
            }
        }

        let client_id = authenticate_client(&stream).await?;
        let client_ip = ip_pool.assign_ip(&client_id).await?;
        tracing::info!("Assigned IP {client_ip} to client {client_id}");

        tunnel_traffic(&stream, client_ip).await?;

        ip_pool.release_ip(&client_id).await?;
        tracing::info!("IP for client {client_id} released");

        Ok(())
    }

    pub async fn run(&self) -> ServerResult<()> {
        let hostname = self.config.hostname();
        tracing::info!("Server listening at {hostname}");
        loop {
            let hostname = hostname.clone();
            let (mut stream, _) = self.listener.accept().await?;
            let permit = self.client_limit.clone().acquire_owned().await?;
            let ip_pool = Arc::clone(&self.ip_pool);

            tokio::spawn(async move {
                if let Err(e) = Server::handle_client(stream, ip_pool, hostname).await {
                    tracing::error!("Error handling client: {e}");
                }

                drop(permit);
            });
        }
    }
}
