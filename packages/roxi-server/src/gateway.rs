use crate::{error::ServerError, ServerResult};
use async_std::sync::Arc;
use roxi_client::Config;
use roxi_lib::types::{ClientId, InterfaceKind};
use roxi_proto::{
    command, Message, MessageKind, MessageStatus, WireGuardProtoConfig,
    WireGuardProtoPeer,
};
use std::collections::HashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock, Semaphore},
    time::{timeout, Duration},
};

pub struct Gateway {
    tcp: TcpListener,
    client_limit: Arc<Semaphore>,
    config: Config,
    wireguard_config: Arc<Mutex<WireGuardProtoConfig>>,
    client_streams: Arc<RwLock<HashMap<ClientId, Arc<Mutex<TcpStream>>>>>,
}

impl Gateway {
    pub async fn new(config: Config) -> ServerResult<Self> {
        let tcp = TcpListener::bind(config.gateway_addr(InterfaceKind::Tcp)).await?;
        let client_limit =
            Arc::new(Semaphore::new(config.max_gateway_clients() as usize));

        let wireguard_config = WireGuardProtoConfig::try_from(config.wireguard())?;
        Ok(Self {
            tcp,
            client_limit,
            config: config.clone(),
            wireguard_config: Arc::new(Mutex::new(wireguard_config)),
            client_streams: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn handle_conn(&self, stream: TcpStream) -> ServerResult<()> {
        tracing::info!("Handling incoming tcp stream");

        let client_id = ClientId::try_from(&stream)?;
        let stream = Arc::new(Mutex::new(stream));

        loop {
            let mut buff = vec![0u8; 1024];
            let n = stream.lock().await.read(&mut buff).await?;
            if n == 0 {
                tracing::warn!("{client_id:?} connection closed");
                break;
            }

            tracing::info!("Read {n} bytes");

            let msg = Message::deserialize(&buff)?;

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
                MessageKind::PeerTunnelInitRequest => {
                    let peer: WireGuardProtoPeer = bincode::deserialize(&msg.data())?;
                    self.wireguard_config.lock().await.add_peer(peer);
                    self.wireguard_config
                        .lock()
                        .await
                        .save(self.config.wireguard_filepath())?;

                    let pubkey = command::cat_wireguard_pubkey()?;

                    let allowed_ips = "".to_string();
                    let endpoint = None;
                    let persistent_keepalive = 1;
                    let data = bincode::serialize(&WireGuardProtoPeer {
                        public_key: pubkey,
                        allowed_ips,
                        endpoint,
                        persistent_keepalive: Some(persistent_keepalive),
                    })?;

                    self.send(
                        &client_id,
                        Message::new(
                            MessageKind::PeerTunnelInitResponse,
                            MessageStatus::r#Ok,
                            self.config.stun_addr().expect("STUN address required"),
                            Some(data),
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
                                MessageStatus::Unauthorized,
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
        }

        Ok(())
    }

    pub async fn stop(self: Arc<Self>) -> ServerResult<()> {
        if let Err(e) = self.stop_with_timeout(Duration::from_secs(1)).await {
            tracing::error!("Error stopping server: {e}");
        }
        Ok(())
    }

    async fn stop_with_timeout(self: Arc<Self>, duration: Duration) -> ServerResult<()> {
        if let Err(e) = timeout(duration, self.stop_inner()).await? {
            tracing::error!("Server shutdown timed out: {e}");
        }
        Ok(())
    }

    async fn stop_inner(&self) -> ServerResult<()> {
        tracing::info!("Initiating graceful server shutdown");

        {
            let mut clients = self.client_streams.write().await;
            for (client_id, stream) in clients.iter() {
                tracing::info!("Closing connection for client: {:?}", client_id);

                // FIXME: Trying to acquire a lock on the stream here causes lock contention due to
                // `Server::handle_conn` accessing the lock infinitum, which (for now) allows clients
                // to send multiple messages on a single stream. Might require a bit of rework later. <( '.' )>
                let mut guard = stream.lock().await;

                if let Err(e) = timeout(
                    Duration::from_secs(self.config.response_timeout()),
                    self.send(
                        client_id,
                        Message::new(
                            MessageKind::ServerShutdown,
                            MessageStatus::ServiceUnavailable,
                            self.config.remote_addr(InterfaceKind::Tcp),
                            None,
                        ),
                        stream.clone(),
                    ),
                )
                .await
                {
                    tracing::error!(
                        "{client_id:?} MessageKind::ServerShutdown timed out: {e}"
                    );
                }

                let _ = AsyncWriteExt::shutdown(&mut *guard).await;
            }
            clients.clear();
        }

        drop(self.client_limit.clone());

        tracing::info!("Server shutdown complete");
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
