use crate::{config::Config, error::ServerError, session::SessionManager, ServerResult};
use async_std::sync::Arc;
use roxi_lib::types::{ClientId, SharedKey};
use roxi_proto::{Message, MessageKind};
use std::collections::HashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, Semaphore},
};

pub struct Server {
    listener: TcpListener,
    client_limit: Arc<Semaphore>,
    config: Config,
    #[allow(unused)]
    clients: Arc<Mutex<HashMap<ClientId, TcpStream>>>,
    sessions: SessionManager,
}

impl Server {
    pub async fn new(config: Config) -> ServerResult<Self> {
        let listener = TcpListener::bind(config.hostname()).await?;
        let client_limit = Arc::new(Semaphore::new(config.max_clients()));

        Ok(Self {
            listener,
            client_limit,
            config: config.clone(),
            clients: Arc::new(Mutex::new(HashMap::new())),
            sessions: SessionManager::new(config),
        })
    }

    pub async fn handle_client(&self, mut stream: TcpStream) -> ServerResult<()> {
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
                let msg = Message::new(MessageKind::Pong, self.config.hostname(), None);
                let data = msg.serialize()?;
                stream.write_all(&data).await?;
            }
            MessageKind::AuthenticationRequest => {
                let key = SharedKey::try_from(msg.data())?;
                let client_id = ClientId::try_from(&stream)?;
                if let Err(_e) = self.sessions.authenticate(&client_id, &key).await {
                    return Err(ServerError::Unauthenticated);
                }
            }
            _ => {
                return Err(ServerError::InvalidMessage);
            }
        }

        Ok(())
    }

    pub async fn run(self: Arc<Self>) -> ServerResult<()> {
        tracing::info!("Server listening at {}", self.config.hostname());
        loop {
            let (stream, _) = self.listener.accept().await?;
            let permit = self.client_limit.clone().acquire_owned().await?;
            let server = Arc::clone(&self);

            tokio::spawn(async move {
                if let Err(e) = server.handle_client(stream).await {
                    tracing::error!("Error handling client: {e}");
                }

                drop(permit);
            });
        }
    }
}
