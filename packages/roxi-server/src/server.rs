use crate::{config::Config, error::ServerError, session::SessionManager, ServerResult};
use async_std::sync::Arc;
use roxi_lib::types::{ClientId, SharedKey, StunAddressKind, StunInfo};
use roxi_proto::{Message, MessageKind};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{RwLock, Semaphore},
};

const STUN_BINDING_REQUEST: u16 = 0x0001;

pub struct Server {
    tcp: TcpListener,
    udp: UdpSocket,
    client_limit: Arc<Semaphore>,
    config: Config,
    client_streams: Arc<RwLock<HashMap<ClientId, TcpStream>>>,
    client_sessions: SessionManager,
    client_stun: Arc<RwLock<HashMap<ClientId, StunInfo>>>,
}

impl Server {
    pub async fn new(config: Config) -> ServerResult<Self> {
        let tcp = TcpListener::bind(config.addr()).await?;
        let udp = UdpSocket::bind(config.addr()).await?;
        let client_limit = Arc::new(Semaphore::new(config.max_clients() as usize));

        Ok(Self {
            tcp,
            udp,
            client_limit,
            config: config.clone(),
            client_streams: Arc::new(RwLock::new(HashMap::new())),
            client_sessions: SessionManager::new(config),
            client_stun: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn handle_tcp(&self, mut stream: TcpStream) -> ServerResult<()> {
        tracing::info!("Handling incoming tcp stream");

        let mut buff = vec![0u8; 1024];
        let n = stream.read(&mut buff).await?;
        if n == 0 {
            return Err(ServerError::ConnectionClosed);
        }

        tracing::info!("Read {n} bytes");

        let msg = Message::deserialize(&buff)?;
        let client_id = ClientId::try_from(&stream)?;

        tracing::info!("Received message from {client_id:?}: {msg:?}");

        match msg.kind() {
            MessageKind::Ping => {
                let msg = Message::new(MessageKind::Pong, self.config.addr(), None);

                tracing::info!("Sending message to {client_id:?}: {msg:?}");
                let data = msg.serialize()?;
                stream.write_all(&data).await?;
            }
            MessageKind::AuthenticationRequest => {
                let key = SharedKey::try_from(msg.data())?;
                if let Err(_e) = self.client_sessions.authenticate(&client_id, &key).await
                {
                    return Err(ServerError::Unauthenticated);
                }

                let msg = Message::new(
                    MessageKind::AuthenticationResponse,
                    self.config.addr(),
                    Some(vec![1]),
                );

                tracing::info!("Sending message to {client_id:?}: {msg:?}");
                let data = msg.serialize()?;
                stream.write_all(&data).await?;

                self.client_streams.write().await.insert(client_id, stream);
            }
            _ => {
                return Err(ServerError::InvalidMessage);
            }
        }

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
        tracing::info!("UDP server listening at {}", self.config.interface());
        let mut buff = [0u8; 1024];

        loop {
            if let Ok((len, addr)) = self.udp.recv_from(&mut buff).await {
                let server = Arc::clone(&self);
                let buff = buff[..len].to_vec();
                tokio::spawn(async move {
                    let _ = server.handle_stun(&buff, addr).await;
                });
            }
        }
    }

    pub async fn run(self: Arc<Self>) -> ServerResult<()> {
        tracing::info!("TCP server listening at {}", self.config.interface());

        let server = Arc::clone(&self);
        tokio::spawn(async move {
            server.client_sessions.monitor_sessions().await;
        });

        loop {
            let (stream, _) = self.tcp.accept().await?;
            tracing::info!("New connection from {:?}", stream.peer_addr());
            let permit = self.client_limit.clone().acquire_owned().await?;
            let server = Arc::clone(&self);

            tokio::spawn(async move {
                if let Err(e) = server.handle_tcp(stream).await {
                    tracing::error!("Error handling client: {e}");
                }

                drop(permit);
            });
        }
    }
}
