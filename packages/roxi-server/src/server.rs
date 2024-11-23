use crate::{config::Config, error::ServerError, session::SessionManager, ServerResult};
use async_std::sync::Arc;
use roxi_client::Config as ClientConfig;
use roxi_lib::types::{ClientId, InterfaceKind, StunAddressKind, StunInfo};
use roxi_proto::{Message, MessageKind, MessageStatus};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{Mutex, RwLock, Semaphore},
};

const STUN_BINDING_REQUEST: u16 = 0x0001;

pub struct Server {
    tcp: TcpListener,
    udp: UdpSocket,
    client_limit: Arc<Semaphore>,
    config: Config,
    client_streams: Arc<RwLock<HashMap<ClientId, Arc<Mutex<TcpStream>>>>>,
    client_sessions: SessionManager,
    client_stun: Arc<RwLock<HashMap<ClientId, StunInfo>>>,
}

impl Server {
    pub async fn new(config: Config) -> ServerResult<Self> {
        Ok(Self {
            tcp: TcpListener::bind(config.addr(InterfaceKind::Tcp)).await?,
            udp: UdpSocket::bind(config.addr(InterfaceKind::Udp)).await?,
            client_limit: Arc::new(Semaphore::new(config.max_clients().into())),
            config: config.clone(),
            client_streams: Arc::new(RwLock::new(HashMap::new())),
            client_sessions: SessionManager::new(config),
            client_stun: Arc::new(RwLock::new(HashMap::new())),
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
                        self.config.remote_addr(InterfaceKind::Tcp),
                        None,
                    ),
                    stream.clone(),
                )
                .await?;
            }
            MessageKind::AuthenticationRequest => {
                let config = ClientConfig::try_from(msg.data())?;
                if let Err(_e) =
                    self.client_sessions.authenticate(&client_id, &config).await
                {
                    self.send(
                        &client_id,
                        Message::new(
                            MessageKind::AuthenticationResponse,
                            MessageStatus::Unauthorized,
                            self.config.remote_addr(InterfaceKind::Tcp),
                            None,
                        ),
                        stream.clone(),
                    )
                    .await?;
                    return Err(ServerError::Unauthenticated);
                }

                self.send(
                    &client_id,
                    Message::new(
                        MessageKind::AuthenticationResponse,
                        MessageStatus::r#Ok,
                        self.config.remote_addr(InterfaceKind::Tcp),
                        None,
                    ),
                    stream.clone(),
                )
                .await?;

                // TODO: Move stream caching into SeedRequest handler
                // as authenticating and becoming a peer should be different actions
                self.client_streams
                    .write()
                    .await
                    .insert(client_id, stream.clone());
            }
            MessageKind::StunInfoRequest => {
                self.ensure_authenticated(
                    &client_id,
                    MessageKind::StunInfoResponse,
                    stream,
                )
                .await?;

                // FIXME: Do we need this?
            }
            MessageKind::GatewayRequest => {
                self.ensure_authenticated(
                    &client_id,
                    MessageKind::GatewayResponse,
                    stream.clone(),
                )
                .await?;

                let peer_addr = self
                    .client_sessions
                    .get_peer_for_gateway(&client_id)
                    .await?;

                let peer_client = ClientId::from(peer_addr.clone());
                tracing::info!(
                    "Peer {peer_client:?} serving GatewayRequest from {client_id:?}"
                );

                let peer_stream = self
                    .client_streams
                    .read()
                    .await
                    .get(&peer_client)
                    .unwrap()
                    .clone();
                self.send(
                    &client_id,
                    Message::new(
                        MessageKind::GatewayResponse,
                        MessageStatus::r#Ok,
                        self.config.remote_addr(InterfaceKind::Tcp),
                        None,
                    ),
                    stream.clone(),
                )
                .await?;

                self.send(
                    &peer_client,
                    Message::new(
                        MessageKind::GatewayResponse,
                        MessageStatus::r#Ok,
                        self.config.remote_addr(InterfaceKind::Tcp),
                        peer_addr.into(),
                    ),
                    peer_stream.clone(),
                )
                .await?;
            }
            MessageKind::SeedRequest => {
                self.ensure_authenticated(
                    &client_id,
                    MessageKind::SeedRequest,
                    stream.clone(),
                )
                .await?;

                // TODO: Remove stream caching from AuthenticationRequest handler,
                // as becoming a seeder and simply authenticating should not be treated
                // as the same action
                self.client_streams
                    .write()
                    .await
                    .insert(client_id.clone(), stream.clone());

                let clients = self.client_streams.read().await;
                tracing::info!("Seeded clients: {:?}", clients);
                self.send(
                    &client_id,
                    Message::new(
                        MessageKind::SeedResponse,
                        MessageStatus::r#Ok,
                        self.config.remote_addr(InterfaceKind::Tcp),
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
                        self.config.remote_addr(InterfaceKind::Tcp),
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

    async fn ensure_authenticated(
        &self,
        client_id: &ClientId,
        kind: MessageKind,
        stream: Arc<Mutex<TcpStream>>,
    ) -> ServerResult<()> {
        if !self.client_sessions.exists(client_id).await {
            tracing::error!("Unauthenticated client: {client_id:?}");
            self.send(
                client_id,
                Message::new(
                    kind,
                    MessageStatus::Unauthorized,
                    self.config.remote_addr(InterfaceKind::Tcp),
                    None,
                ),
                stream,
            )
            .await?;

            return Err(ServerError::Unauthenticated);
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

    pub async fn handle_stun(&self, buff: &[u8], addr: SocketAddr) -> ServerResult<()> {
        let msg_type = u16::from_be_bytes([buff[0], buff[1]]);
        if msg_type != STUN_BINDING_REQUEST {
            tracing::warn!("Unrecognized stun request");
            return Ok(());
        }

        let ip = match addr.ip() {
            IpAddr::V4(ip) => ip,
            _ => {
                tracing::warn!("Only Ipv4 supported at this time");
                return Err(ServerError::UnsupportedIpAddrType);
            }
        };

        let port = addr.port();
        let client_id = ClientId::from(&addr);
        let info = StunInfo::new(StunAddressKind::Public, ip, port);

        tracing::info!("Adding stun info for {client_id:?}: {info:?}");
        self.client_stun.write().await.insert(client_id, info);

        Ok(())
    }

    pub async fn run_stun(self: Arc<Self>) -> ServerResult<()> {
        tracing::info!(
            "UDP server listening at {}",
            self.config.addr(InterfaceKind::Udp)
        );
        let mut buff = [0u8; 1024];

        loop {
            if let Ok((len, addr)) = self.udp.recv_from(&mut buff).await {
                let server = Arc::clone(&self);
                let buff = buff[..len].to_vec();
                tokio::spawn(async move {
                    tracing::info!("New UDP packets");
                    let _ = server.handle_stun(&buff, addr).await;
                });
            }
        }
    }

    pub async fn run(self: Arc<Self>) -> ServerResult<()> {
        tracing::info!(
            "Roxi server listening at {}",
            self.config.addr(InterfaceKind::Tcp)
        );

        let server = Arc::clone(&self);
        tokio::spawn(async move {
            server.client_sessions.monitor().await;
        });

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

    pub async fn stop(self: Arc<Self>) -> ServerResult<()> {
        tracing::info!("Initiating graceful server shutdown");

    // Close all existing client TCP connections
    {
        let mut clients = self.client_streams.write().await;
        for (client_id, stream) in clients.iter() {
            tracing::info!("Closing connection for client: {:?}", client_id);
            let mut stream_guard = stream.lock().await;
            
            // Try to send shutdown message to client
            let _ = self.send(
                client_id,
                Message::new(
                    MessageKind::ServerShutdown,
                    MessageStatus::ServiceUnavailable,
                    self.config.remote_addr(InterfaceKind::Tcp),
                    None,
                ),
                stream.clone(),
            ).await;
            
            // Force close the stream using AsyncWriteExt::shutdown
            let _ = AsyncWriteExt::shutdown(&mut *stream_guard).await;
        }
        clients.clear();
    }
    
        // Clean up session state
        self.client_sessions.clear().await?;
    
        // Clear STUN information
        self.client_stun.write().await.clear();
    
        // Drop the semaphore to prevent new connections
        drop(self.client_limit.clone());
    
        tracing::info!("Server shutdown complete");
        Ok(())
    }
}
