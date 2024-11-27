use crate::{config::Config, ClientResult};
use bytes::BytesMut;
use roxi_lib::types::{Address, ClientId, InterfaceKind};
use roxi_proto::{
    command, Message, MessageKind, MessageStatus, WireGuardProtoConfig,
    WireGuardProtoPeer,
};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UdpSocket},
    sync::Mutex,
    time::{sleep, timeout, Duration},
};

const STUN_BINDING_REQUEST_TYPE: u16 = 0x0001;
const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

pub struct Client {
    config: Config,
    wireguard_config: Arc<Mutex<WireGuardProtoConfig>>,
    tcp: TcpStream,
    udp: UdpSocket,
    peer_stream: Option<(ClientId, Address, Arc<Mutex<TcpStream>>)>,
}

impl Client {
    pub fn client_id(&self) -> ClientId {
        ClientId::from(self.config.gateway_remote_addr(InterfaceKind::Tcp))
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn new(config: Config) -> ClientResult<Self> {
        let wireguard_config = WireGuardProtoConfig::try_from(config.wireguard())?;

        Ok(Self {
            config: config.clone(),
            wireguard_config: Arc::new(Mutex::new(wireguard_config)),
            tcp: TcpStream::connect(&config.remote_addr(InterfaceKind::Tcp))
                .await
                .expect("Failed to connect to TCP server"),
            udp: UdpSocket::bind(&config.addr(InterfaceKind::Udp))
                .await
                .expect("Failed to bind to UDP socket"),
            peer_stream: None,
        })
    }

    pub async fn ping(&mut self) -> ClientResult<Option<Message>> {
        match self
            .send(Message::new(
                MessageKind::Ping,
                MessageStatus::Pending,
                self.config.remote_addr(InterfaceKind::Tcp),
                None,
            ))
            .await?
        {
            Some(msg) => {
                tracing::info!("Successfully pinged client connection");
                Ok(Some(msg))
            }
            None => {
                tracing::error!("Failed to ping client connection");
                Ok(None)
            }
        }
    }

    pub async fn authenticate(&mut self) -> ClientResult<Option<Message>> {
        let secret = self.config.clone().try_into()?;
        match self
            .send(Message::new(
                MessageKind::AuthenticationRequest,
                MessageStatus::Pending,
                self.config.remote_addr(InterfaceKind::Tcp),
                Some(secret),
            ))
            .await?
        {
            Some(msg) => {
                tracing::info!("Successfully authenticated client connection");
                Ok(Some(msg))
            }
            None => {
                tracing::error!("Failed to authenticate client connection");
                Ok(None)
            }
        }
    }

    pub async fn stun(&mut self) -> ClientResult<()> {
        let mut request = BytesMut::with_capacity(20);
        request.extend_from_slice(&u16::to_be_bytes(STUN_BINDING_REQUEST_TYPE));
        request.extend_from_slice(&u16::to_be_bytes(0)); // Length
        request.extend_from_slice(&u32::to_be_bytes(STUN_MAGIC_COOKIE));

        for _ in 0..12 {
            request.extend_from_slice(&[rand::random::<u8>()]);
        }

        tracing::info!("Sending info to STUN server");
        let send_result = timeout(
            Duration::from_secs(1),
            self.udp
                .send_to(&request, self.config.remote_addr(InterfaceKind::Udp)),
        )
        .await;

        if let Ok(Ok(_)) = send_result {
            tracing::info!("Successfully sent info to STUN server");
        }

        Ok(())
    }

    pub async fn seed(&mut self) -> ClientResult<Option<Message>> {
        self.authenticate().await?;
        match self
            .send(Message::new(
                MessageKind::SeedRequest,
                MessageStatus::Pending,
                self.config.remote_addr(InterfaceKind::Tcp),
                None,
            ))
            .await?
        {
            Some(msg) => {
                tracing::info!("Successfully seeded client connection");
                Ok(Some(msg))
            }
            None => {
                tracing::error!("Failed to seed client connection");
                Ok(None)
            }
        }
    }

    pub async fn request_stun_info(&mut self) -> ClientResult<Option<Message>> {
        match self
            .send(Message::new(
                MessageKind::StunInfoRequest,
                MessageStatus::Pending,
                self.config.remote_addr(InterfaceKind::Tcp),
                None,
            ))
            .await?
        {
            Some(msg) => {
                tracing::info!("Successfully received STUN info");
                Ok(Some(msg))
            }
            None => {
                tracing::error!("Failed to receive STUN info");
                Ok(None)
            }
        }
    }

    pub async fn request_gateway(&mut self) -> ClientResult<Option<Message>> {
        match self
            .send(Message::new(
                MessageKind::GatewayRequest,
                MessageStatus::Pending,
                self.config.remote_addr(InterfaceKind::Tcp),
                None,
            ))
            .await?
        {
            Some(msg) => {
                tracing::info!("Successfully received gateway info");
                Ok(Some(msg))
            }
            None => {
                tracing::error!("Failed to receive gateway info");
                Ok(None)
            }
        }
    }

    pub async fn nat_punch(&mut self, addr: Address) -> ClientResult<()> {
        tracing::info!("Attempting NAT punch to {addr:?}");

        // TODO: What is the timeout on this?
        match TcpStream::connect(&addr.to_string()).await {
            Ok(stream) => {
                tracing::info!(
                    "NAT punch received response. Connection to {addr:?} is open!"
                );
                let client_id = ClientId::from(addr.clone());
                self.peer_stream = Some((client_id, addr, Arc::new(Mutex::new(stream))));
            }
            Err(e) => {
                tracing::info!("NAT punch failed. Will try again: {e}");
                let max_attempts = self.config.nat_punch_attempts();
                let mut attempts = 0;
                while attempts < max_attempts {
                    if let Err(e) = TcpStream::connect(&addr.to_string()).await {
                        tracing::warn!(
                            "NAT punch attempt {}/{} failed: {}",
                            attempts,
                            attempts + 1,
                            e
                        );
                        attempts += 1;
                    }
                }

                if attempts == max_attempts {
                    tracing::error!("NAT punch max attempts reached");
                    return Ok(());
                }
            }
        }

        tracing::info!("NAT punch attempt(s) finished.");
        sleep(Duration::from_secs(self.config.nat_punch_delay().into())).await;
        Ok(())
    }

    async fn request_tunnel_info(&mut self) -> ClientResult<Option<Message>> {
        let pubkey = command::cat_wireguard_pubkey()?;
        let endpoint = None;
        let allowed_ips = "".to_string();
        let persistent_keepalive = 1;

        let data = bincode::serialize(&WireGuardProtoPeer {
            public_key: pubkey.to_owned(),
            allowed_ips,
            endpoint,
            persistent_keepalive: Some(persistent_keepalive),
        })?;

        match self
            .send(Message::new(
                MessageKind::PeerTunnelInitRequest,
                MessageStatus::Pending,
                self.config.remote_addr(InterfaceKind::Tcp),
                Some(data),
            ))
            .await?
        {
            Some(msg) => {
                tracing::info!("Received tunnel info: {msg:?}");
                let peer: WireGuardProtoPeer = bincode::deserialize(&msg.data())?;
                self.wireguard_config.lock().await.add_peer(peer);

                //            self.wireguard_config.save();

                Ok(Some(msg))
            }
            None => {
                tracing::error!("Failed to receive tunnel info");
                Ok(None)
            }
        }
    }

    pub async fn setup_peer_tunnel(&mut self, addr: Address) -> ClientResult<()> {
        let _msg = self
            .send(Message::new(
                MessageKind::PeerTunnelRequest,
                MessageStatus::Pending,
                self.config.remote_addr(InterfaceKind::Tcp),
                addr.into(),
            ))
            .await?;

        Ok(())
    }

    pub async fn tunnel(&mut self) -> ClientResult<()> {
        self.authenticate().await?;
        let msg = self.request_gateway().await?;
        if let Some(msg) = msg {
            let addr = Address::try_from(msg.data())?;
            if let Err(e) = timeout(
                Duration::from_secs(self.config.request_timeout()),
                self.nat_punch(addr.clone()),
            )
            .await
            {
                tracing::error!("NAT punch failed: {e}");
                return Ok(());
            }
            self.request_tunnel_info().await?;
            self.setup_peer_tunnel(addr).await?;
        }
        Ok(())
    }

    async fn send(&mut self, m: Message) -> ClientResult<Option<Message>> {
        tracing::info!("Sending message: {m:?}");
        let data = m.serialize()?;
        tracing::info!("Sending {} bytes", data.len());

        match timeout(
            Duration::from_secs(self.config.request_timeout()),
            self.tcp.write_all(&data),
        )
        .await
        {
            Ok(_) => {
                let mut buff = vec![0u8; 1024];
                let n = self.tcp.read(&mut buff).await?;
                if n > 0 {
                    let data = buff[..n].to_vec();
                    let msg = Message::deserialize(&data)?;
                    tracing::info!("Received response: {msg:?}");
                    match msg.status() {
                        MessageStatus::r#Ok | MessageStatus::Created => {
                            tracing::info!("Recevied successful response");
                        }
                        _ => {
                            tracing::warn!("Received non-success response");
                        }
                    }
                    return Ok(Some(msg));
                }

                tracing::info!("No data in response");
            }
            Err(e) => {
                tracing::error!("Request timeout: {e}");
            }
        }

        Ok(None)
    }

    pub async fn stop(&mut self) -> ClientResult<()> {
        if let Err(e) = self.stop_with_timeout(Duration::from_secs(1)).await {
            tracing::error!("Error stopping server: {e}");
        }
        Ok(())
    }

    async fn stop_with_timeout(&mut self, duration: Duration) -> ClientResult<()> {
        if let Err(e) = timeout(duration, self.stop_inner()).await? {
            tracing::error!("Server shutdown timed out: {e}");
        }
        Ok(())
    }

    pub async fn stop_inner(&mut self) -> ClientResult<()> {
        tracing::info!("Stopping client");
        if let Some((client_id, addr, stream)) = self.peer_stream.take() {
            let _msg = self
                .send(Message::new(
                    MessageKind::PeerTunnelClose,
                    MessageStatus::Pending,
                    addr.to_string(),
                    Some(client_id.to_vec()),
                ))
                .await?;

            stream.lock().await.shutdown().await?;
        }

        Ok(())
    }
}
