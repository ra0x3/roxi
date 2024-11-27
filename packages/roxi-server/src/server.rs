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
    time::{timeout, Duration},
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
    pub fn config(&self) -> &Config {
        &self.config
    }

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

    pub async fn handle_conn(&self, stream: TcpStream) -> ServerResult<()> {
        tracing::info!("Handling incoming tcp stream");

        let client_id = ClientId::try_from(&stream)?;
        let stream = Arc::new(Mutex::new(stream));

        loop {
            let mut buff = vec![0u8; 1024];
            let n = stream.lock().await.read(&mut buff).await?;
            if n == 0 {
                tracing::warn!("Client connection closed");
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
                        .insert(client_id.clone(), stream.clone());
                }
                MessageKind::StunInfoRequest => {
                    self.ensure_authenticated(
                        &client_id,
                        MessageKind::StunInfoResponse,
                        stream.clone(),
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
                        MessageKind::SeedResponse,
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
        let _ = stream.lock().await.write_all(&data).await;
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

    /// Runs the STUN server, processing incoming UDP packets asynchronously.
    ///
    /// This method listens on the UDP socket specified in the server's configuration.
    /// For each incoming packet, it spawns a new task to process the packet using
    /// `handle_stun`. The server instance is shared across tasks using `Arc<Self>`.
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

    /// Runs the TCP server, accepting and handling client connections asynchronously.
    ///
    /// This method listens on the TCP socket specified in the server's configuration.
    /// For each incoming connection, it spawns a new task to process the connection
    /// using `handle_conn`. The server instance is shared across tasks using `Arc<Self>`.
    ///
    /// Additionally, this method spawns a background task to monitor client sessions
    /// using `client_sessions.monitor`.
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

        self.client_sessions.clear().await?;
        self.client_stun.write().await.clear();

        drop(self.client_limit.clone());

        tracing::info!("Server shutdown complete");
        Ok(())
    }
}
