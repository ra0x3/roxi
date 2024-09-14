use crate::{error::ServerError, ServerResult};
use async_std::sync::Arc;
use roxi_client::Config;
use roxi_lib::types::{ClientId, InterfaceKind};
use roxi_proto::{Message, MessageKind, MessageStatus};
use std::collections::HashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock, Semaphore},
};

pub struct Gateway {
    tcp: TcpListener,
    client_limit: Arc<Semaphore>,
    config: Config,
    #[allow(unused)]
    client_streams: Arc<RwLock<HashMap<ClientId, Arc<Mutex<TcpStream>>>>>,
}

impl Gateway {
    pub async fn new(config: Config) -> ServerResult<Self> {
        let tcp = TcpListener::bind(config.gateway_addr(InterfaceKind::Tcp)).await?;
        let client_limit =
            Arc::new(Semaphore::new(config.max_gateway_clients() as usize));

        Ok(Self {
            tcp,
            client_limit,
            config: config.clone(),
            client_streams: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn handle_conn(&self, mut stream: TcpStream) -> ServerResult<()> {
        tracing::info!("Handling incoming tcp stream");

        let mut buff = vec![0u8; 1024];
        let n = stream.read(&mut buff).await?;
        if n == 0 {
            tracing::warn!("Client connection closed");
            return Err(ServerError::ConnectionClosed);
        }

        tracing::info!("Read {n} bytes");

        let msg = Message::deserialize(&buff)?;
        let client_id = ClientId::try_from(&stream)?;
        let stream = Arc::new(Mutex::new(stream));

        tracing::info!("Received message from {client_id:?}: {msg:?}");

        match msg.kind() {
            MessageKind::Ping => {
                self.send(
                    &client_id,
                    Message::new(
                        MessageKind::Pong,
                        MessageStatus::r#Ok,
                        self.config.stun_addr().expect("STUN address required"),
                        None,
                    ),
                    stream.clone(),
                )
                .await?;
            }
            MessageKind::PeerTunnelRequest => {
                self.client_streams
                    .write()
                    .await
                    .insert(client_id.clone(), stream.clone());

                self.send(
                    &client_id,
                    Message::new(
                        MessageKind::PeerTunnelResponse,
                        MessageStatus::r#Ok,
                        self.config.stun_addr().expect("STUN address required"),
                        None,
                    ),
                    stream.clone(),
                )
                .await?;
            }
            MessageKind::NATPunchRequest => {
                if !self.client_streams.read().await.contains_key(&client_id) {
                    tracing::error!("{client_id:?} not a recognized peer");
                    self.send(
                        &client_id,
                        Message::new(
                            MessageKind::NATPunchResponse,
                            MessageStatus::Forbidden,
                            self.config.stun_addr().expect("STUN address required"),
                            None,
                        ),
                        stream.clone(),
                    )
                    .await?;
                }

                self.send(
                    &client_id,
                    Message::new(
                        MessageKind::NATPunchResponse,
                        MessageStatus::r#Ok,
                        self.config.stun_addr().expect("STUN address required"),
                        None,
                    ),
                    stream.clone(),
                )
                .await?;
            }
            _ => {
                self.send(
                    &client_id,
                    Message::new(
                        MessageKind::GenericErrorResponse,
                        MessageStatus::BadData,
                        self.config.stun_addr().expect("STUN address required"),
                        None,
                    ),
                    stream.clone(),
                )
                .await?;
                return Err(ServerError::InvalidMessage);
            }
        }

        Ok(())
    }

    async fn send(
        &self,
        client_id: &ClientId,
        msg: Message,
        stream: Arc<Mutex<TcpStream>>,
    ) -> ServerResult<()> {
        tracing::info!("Sending message to {client_id:?}: {msg:?}");
        let data = msg.serialize()?;
        stream.lock().await.write_all(&data).await?;
        Ok(())
    }

    pub async fn run(self: Arc<Self>) -> ServerResult<()> {
        tracing::info!(
            "Gateway server listening at {}",
            self.config.gateway_addr(InterfaceKind::Tcp)
        );

        loop {
            let (stream, _) = self.tcp.accept().await?;
            tracing::info!("New connection from {:?}", stream.peer_addr());
            let permit = self.client_limit.clone().acquire_owned().await?;
            let server = Arc::clone(&self);

            tokio::spawn(async move {
                if let Err(e) = server.handle_conn(stream).await {
                    tracing::error!("Error handling client: {e}");
                }

                drop(permit);
            });
        }
    }
}
