use crate::{config::Config, ClientResult};
use bytes::BytesMut;
use roxi_lib::types::{Address, ClientId, InterfaceKind};
use roxi_proto::{Message, MessageKind, MessageStatus};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UdpSocket},
    sync::{Mutex, RwLock},
    time::{sleep, Duration},
};

const STUN_BINDING_REQUEST_TYPE: u16 = 0x0001;
const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

pub struct Client {
    config: Config,
    tcp: TcpStream,
    udp: UdpSocket,
    peer_streams: Arc<RwLock<HashMap<ClientId, Arc<Mutex<TcpStream>>>>>,
}

impl Client {
    pub async fn new(config: Config) -> ClientResult<Self> {
        let tcp = TcpStream::connect(&config.remote_addr(InterfaceKind::Tcp)).await?;
        let udp = UdpSocket::bind(&config.remote_addr(InterfaceKind::Udp)).await?;
        let peer_streams = Arc::new(RwLock::new(HashMap::new()));
        Ok(Self {
            config,
            tcp,
            udp,
            peer_streams,
        })
    }

    pub async fn connect(&mut self) -> ClientResult<()> {
        self.authenticate().await?;
        self.stun().await?;
        if let Some(addr) = self.request_gateway().await? {
            self.nat_punch(addr).await?;
        }

        Ok(())
    }

    pub async fn ping(&mut self) -> ClientResult<()> {
        let _msg = self
            .send(Message::new(
                MessageKind::Ping,
                MessageStatus::Pending,
                self.config.stun_addr()?,
                None,
            ))
            .await?;

        Ok(())
    }

    pub async fn authenticate(&mut self) -> ClientResult<()> {
        let data = self.config.clone().try_into()?;
        let _msg = self
            .send(Message::new(
                MessageKind::AuthenticationRequest,
                MessageStatus::Pending,
                self.config.stun_addr()?,
                Some(data),
            ))
            .await?;

        Ok(())
    }

    pub async fn stun(&mut self) -> ClientResult<()> {
        let mut request = BytesMut::with_capacity(20);
        request.extend_from_slice(&u16::to_be_bytes(STUN_BINDING_REQUEST_TYPE));
        request.extend_from_slice(&u16::to_be_bytes(0)); // Length
        request.extend_from_slice(&u32::to_be_bytes(STUN_MAGIC_COOKIE));

        // Add transaction ID
        for _ in 0..12 {
            request.extend_from_slice(&[rand::random::<u8>()]);
        }

        tracing::info!("Sending info to STUN server");
        self.udp
            .send_to(&request, self.config.remote_addr(InterfaceKind::Udp))
            .await?;

        tracing::info!("Successfully sent info to STUN server");

        Ok(())
    }

    pub async fn request_stun_info(&mut self) -> ClientResult<()> {
        let _msg = self
            .send(Message::new(
                MessageKind::StunInfoRequest,
                MessageStatus::Pending,
                self.config.remote_addr(InterfaceKind::Tcp),
                None,
            ))
            .await?;

        Ok(())
    }

    pub async fn request_gateway(&mut self) -> ClientResult<Option<Address>> {
        let msg = self
            .send(Message::new(
                MessageKind::GatewayRequest,
                MessageStatus::Pending,
                self.config.remote_addr(InterfaceKind::Tcp),
                None,
            ))
            .await?;

        let data = msg.expect("Empty response").into_inner();
        let addr = Address::try_from(data)?;

        Ok(Some(addr))
    }

    async fn nat_punch(&mut self, addr: Address) -> ClientResult<()> {
        tracing::info!("Attempting NAT punch to {addr:?}");

        // FIXME: What is the timeout on this?
        match TcpStream::connect(&addr.to_string()).await {
            Ok(stream) => {
                tracing::info!(
                    "NAT punch received response. Connection to {addr:?} is open!"
                );
                self.peer_streams
                    .write()
                    .await
                    .insert(ClientId::from(addr), Arc::new(Mutex::new(stream)));
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

    #[allow(unused)]
    async fn request_tunnel_info(&mut self) -> ClientResult<()> {
        Ok(())
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

    async fn send(&mut self, m: Message) -> ClientResult<Option<Message>> {
        tracing::info!("Sending message: {m:?}");
        let data = m.serialize()?;
        tracing::info!("Sending {} bytes", data.len());

        self.tcp.write_all(&data).await?;
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

        Ok(None)
    }
}
