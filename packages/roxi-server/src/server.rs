use crate::{config::Config, error::ServerError, session::SessionManager, ServerResult};
use async_std::sync::Arc;
use roxi_lib::types::{ClientId, SharedKey};
use roxi_proto::{Message, MessageKind};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::{RwLock, Semaphore},
};

const STUN_BINDING_RESPONSE: u16 = 0x0101;
const MAPPED_ADDRESS_ATTRIBUTE_TYPE: u16 = 0x0001;
const STUN_BINDING_REQUEST: u16 = 0x0001;
const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

pub struct Server {
    listener: TcpListener,
    udp_listener: UdpSocket,
    client_limit: Arc<Semaphore>,
    config: Config,
    clients: Arc<RwLock<HashMap<ClientId, TcpStream>>>,
    sessions: SessionManager,
}

impl Server {
    pub async fn new(config: Config) -> ServerResult<Self> {
        let listener = TcpListener::bind(config.hostname()).await?;
        let udp_listener = UdpSocket::bind(config.hostname()).await?;
        let client_limit = Arc::new(Semaphore::new(config.max_clients() as usize));

        Ok(Self {
            listener,
            udp_listener,
            client_limit,
            config: config.clone(),
            clients: Arc::new(RwLock::new(HashMap::new())),
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

                self.clients.write().await.insert(client_id, stream);
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

        let tx_id = &buff[8..20];

        let mut response = bytes::BytesMut::with_capacity(32);
        response.extend_from_slice(&u16::to_be_bytes(STUN_BINDING_RESPONSE));
        response.extend_from_slice(&u16::to_be_bytes(12)); // Length
        response.extend_from_slice(&u32::to_be_bytes(STUN_MAGIC_COOKIE));
        response.extend_from_slice(tx_id);

        response.extend_from_slice(&u16::to_be_bytes(MAPPED_ADDRESS_ATTRIBUTE_TYPE));
        response.extend_from_slice(&u16::to_be_bytes(8)); // Length
        response.extend_from_slice(&[0, 1]); // IPv4 family
        response.extend_from_slice(&u16::to_be_bytes(
            addr.port() ^ (STUN_MAGIC_COOKIE >> 16) as u16,
        )); // XOR-mapped port
        let ip = match addr.ip() {
            std::net::IpAddr::V4(ip) => ip.octets(),
            _ => return Ok(()),
        };
        for i in 0..4 {
            response.extend_from_slice(&[ip[i] ^ (STUN_MAGIC_COOKIE.to_be_bytes()[i])]);
            // XOR-mapped IP
        }
        Ok(())
    }

    pub async fn run_stun(self: Arc<Self>) -> ServerResult<()> {
        tracing::info!("UDP server listening at {}", self.config.hostname());
        let server = Arc::clone(&self);
        let mut buff = [0u8; 1024];

        loop {
            if let Ok((len, addr)) = self.udp_listener.recv_from(&mut buff).await {
                let server = Arc::clone(&self);
                let buf = buff[..len].to_vec();
                tokio::spawn(async move {
                    let _ = server.handle_stun(&buff, addr).await;
                });
            }
        }
    }

    pub async fn run(self: Arc<Self>) -> ServerResult<()> {
        tracing::info!("TCP server listening at {}", self.config.hostname());

        let server = Arc::clone(&self);
        tokio::spawn(async move {
            server.sessions.monitor_sessions().await;
        });

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
