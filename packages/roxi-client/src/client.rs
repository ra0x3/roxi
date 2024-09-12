use crate::{config::Config, ClientResult};
use bytes::BytesMut;
use roxi_lib::types::Address;
use roxi_proto::{Message, MessageKind, MessageStatus};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UdpSocket},
};

const STUN_BINDING_REQUEST_TYPE: u16 = 0x0001;
const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

pub struct Client {
    config: Config,
    tcp: TcpStream,
    udp: UdpSocket,
}

impl Client {
    pub async fn new(config: Config) -> ClientResult<Self> {
        let tcp = TcpStream::connect(&config.remote_addr()).await?;
        let udp = UdpSocket::bind(&config.udp_bind()).await?;
        Ok(Self { config, tcp, udp })
    }

    pub async fn connect(&mut self) -> ClientResult<()> {
        self.authenticate().await?;
        self.stun().await?;

        Ok(())
    }

    pub async fn ping(&mut self) -> ClientResult<()> {
        let _msg = self
            .send(Message::new(
                MessageKind::Ping,
                MessageStatus::Pending,
                self.config.remote_addr(),
                None,
            ))
            .await?;

        Ok(())
    }

    pub async fn authenticate(&mut self) -> ClientResult<()> {
        let data = self.config.shared_key().try_into()?;
        let _msg = self
            .send(Message::new(
                MessageKind::AuthenticationRequest,
                MessageStatus::Pending,
                self.config.remote_addr(),
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
            .send_to(&request, self.config.remote_addr())
            .await?;

        tracing::info!("Successfully sent info to STUN server");

        Ok(())
    }

    pub async fn request_stun_info(&mut self) -> ClientResult<()> {
        let _msg = self
            .send(Message::new(
                MessageKind::StunInfoRequest,
                MessageStatus::Pending,
                self.config.remote_addr(),
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
                self.config.remote_addr(),
                None,
            ))
            .await?;

        let data = msg.expect("Empty response").into_inner();
        let addr = Address::try_from(data)?;

        let _msg = self
            .send(Message::new(
                MessageKind::NATPunchRequest,
                MessageStatus::Pending,
                self.config.remote_addr(),
                None,
            ))
            .await?;
        Ok(Some(addr))
    }

    pub async fn setup_peer_tunnel(&mut self, addr: Address) -> ClientResult<()> {
        let _msg = self
            .send(Message::new(
                MessageKind::PeerTunnelRequest,
                MessageStatus::Pending,
                self.config.remote_addr(),
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
