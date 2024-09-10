use crate::{config::Config, ClientResult};
use bytes::BytesMut;
use roxi_proto::{Message, MessageKind};
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
        let tcp = TcpStream::connect(&config.remote_hostname()).await?;
        let udp = UdpSocket::bind(&config.udp_bind()).await?;
        Ok(Self { config, tcp, udp })
    }

    pub async fn connect(&mut self) -> ClientResult<()> {
        self.authenticate().await?;
        self.stun().await?;

        Ok(())
    }

    pub async fn ping(&mut self) -> ClientResult<()> {
        let msg = Message::new(MessageKind::Ping, self.config.remote_hostname(), None);
        tracing::info!("Sending message to server: {msg:?}");
        match self.send(msg).await {
            Ok(Some(msg)) => {
                tracing::info!("Successfully received ping response: {msg:?}");
            }
            Err(e) => {
                tracing::error!("Could not ping server: {e}");
            }
            _ => {
                tracing::error!("Received empty ping response from server");
            }
        }

        Ok(())
    }

    pub async fn authenticate(&mut self) -> ClientResult<()> {
        let data = self.config.shared_key().try_into()?;
        let msg = Message::new(
            MessageKind::AuthenticationRequest,
            self.config.remote_hostname(),
            Some(data),
        );
        tracing::info!("Sending message to server: {msg:?}");
        match self.send(msg).await {
            Ok(Some(msg)) => {
                tracing::info!("Successfully authenticated against server: {msg:?}");
            }
            Err(e) => {
                tracing::error!("Could not authenticate against server: {e}");
            }
            _ => {
                tracing::error!("Recevied empty authentication response from server");
            }
        }

        Ok(())
    }

    pub async fn stun(&mut self) -> ClientResult<()> {
        tracing::info!("Sending info to STUN server");

        let mut request = BytesMut::with_capacity(20);
        request.extend_from_slice(&u16::to_be_bytes(STUN_BINDING_REQUEST_TYPE));
        request.extend_from_slice(&u16::to_be_bytes(0)); // Length
        request.extend_from_slice(&u32::to_be_bytes(STUN_MAGIC_COOKIE));

        // Add transaction ID
        for _ in 0..12 {
            request.extend_from_slice(&[rand::random::<u8>()]);
        }

        self.udp
            .send_to(&request, self.config.remote_hostname())
            .await?;

        tracing::info!("Successfully sent info to STUN server");

        Ok(())
    }

    async fn send(&mut self, m: Message) -> ClientResult<Option<Message>> {
        let data = m.serialize()?;
        tracing::info!("Sending {} bytes", data.len());
        self.tcp.write_all(&data).await?;
        let mut buff = vec![0u8; 1024];
        let n = self.tcp.read(&mut buff).await?;
        if n > 0 {
            let data = buff[..n].to_vec();
            let msg = Message::deserialize(&data)?;
            return Ok(Some(msg));
        }
        Ok(None)
    }
}
