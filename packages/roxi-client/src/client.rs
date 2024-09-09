use crate::{config::Config, error::ClientError, ClientResult};
use bytes::BytesMut;
use roxi_proto::{Message, MessageKind};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UdpSocket},
};

const STUN_BINDING_REQUEST_TYPE: u16 = 0x0001;
const STUN_BINDING_REQUEST: u16 = 0x0101;
const STUN_MAGIC_COOKIE: u32 = 0x2112A442;

pub struct Client {
    config: Config,
    tcp_stream: TcpStream,
    udp_socket: UdpSocket,
    hostname: String,
}

impl Client {
    pub async fn new(config: Config) -> ClientResult<Self> {
        let hostname = config.remote_hostname();
        let tcp_stream = TcpStream::connect(&hostname).await?;
        let udp_socket = UdpSocket::bind(&config.udp_bind()).await?;
        Ok(Self {
            config,
            hostname,
            tcp_stream,
            udp_socket,
        })
    }

    pub async fn connect(&mut self) -> ClientResult<()> {
        let _ = self.authenticate().await?;
        let _ = self.stun().await?;

        Ok(())
    }

    pub async fn ping(&mut self) -> ClientResult<()> {
        let msg = Message::new(MessageKind::Ping, self.hostname.clone(), None);
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

    async fn authenticate(&mut self) -> ClientResult<()> {
        let data = self.config.shared_key().into();
        let msg = Message::new(
            MessageKind::AuthenticationRequest,
            self.hostname.clone(),
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

    async fn stun(&mut self) -> ClientResult<()> {
        let mut request = BytesMut::with_capacity(20);
        request.extend_from_slice(&u16::to_be_bytes(STUN_BINDING_REQUEST_TYPE));
        request.extend_from_slice(&u16::to_be_bytes(0)); // Length
        request.extend_from_slice(&u32::to_be_bytes(STUN_MAGIC_COOKIE));

        // Add transaction ID
        for _ in 0..12 {
            request.extend_from_slice(&[rand::random::<u8>()]);
        }

        self.udp_socket
            .send_to(&request, self.config.remote_hostname())
            .await?;

        /* We don't actuall need any of this
        let mut buff = [0u8; 1024];
        let (len, _) = self.udp_socket.recv_from(&mut buff).await?;

        let msg_type = u16::from_be_bytes([buff[0], buff[1]]);
        if msg_type != STUN_BINDING_REQUEST {
            return Err(ClientError::NotAStunBindingRequest);
        }

        // Extract the Mapped Address from the response
        let port =
            u16::from_be_bytes([buff[28], buff[29]]) ^ (STUN_MAGIC_COOKIE >> 16) as u16;
        let ip = [
            buff[32] ^ STUN_MAGIC_COOKIE.to_be_bytes()[0],
            buff[33] ^ STUN_MAGIC_COOKIE.to_be_bytes()[1],
            buff[34] ^ STUN_MAGIC_COOKIE.to_be_bytes()[2],
            buff[35] ^ STUN_MAGIC_COOKIE.to_be_bytes()[3],
        ];

        println!("Public IP: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
        println!("Public Port: {}", port);
        */
        Ok(())
    }

    async fn send(&mut self, m: Message) -> ClientResult<Option<Message>> {
        let data = m.serialize()?;
        tracing::info!("Sending {} bytes", data.len());
        self.tcp_stream.write_all(&data).await?;
        let mut buff = vec![0u8; 1024];
        let n = self.tcp_stream.read(&mut buff).await?;
        if n > 0 {
            let data = buff[..n].to_vec();
            let msg = Message::deserialize(&data)?;
            return Ok(Some(msg));
        }
        Ok(None)
    }
}
