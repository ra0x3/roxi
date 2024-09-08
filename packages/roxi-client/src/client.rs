use crate::{config::Config, ClientResult};
use roxi_proto::{Message, MessageKind};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Client {
    config: Config,
    stream: TcpStream,
    hostname: String,
}

impl Client {
    pub async fn new(config: Config) -> ClientResult<Self> {
        let hostname = config.roxi_server_hostname();
        let stream = TcpStream::connect(&hostname).await?;
        Ok(Self {
            config,
            hostname,
            stream,
        })
    }

    pub async fn connect(&mut self) -> ClientResult<()> {
        let _ = self.ping().await?;
        let _ = self.authenticate().await?;

        Ok(())
    }

    async fn ping(&mut self) -> ClientResult<()> {
        let m = Message::new(MessageKind::Ping, self.hostname.clone(), None);
        match self.send(m).await {
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
        let m = Message::new(
            MessageKind::AuthenticationRequest,
            self.hostname.clone(),
            Some(data),
        );
        match self.send(m).await {
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

    async fn send(&mut self, m: Message) -> ClientResult<Option<Message>> {
        let data = m.serialize()?;
        self.stream.write_all(&data).await?;
        let mut buff = vec![0u8; 1024];
        let n = self.stream.read(&mut buff).await?;
        if n > 0 {
            let data = buff[..n].to_vec();
            let msg = Message::deserialize(&data)?;
            return Ok(Some(msg));
        }
        Ok(None)
    }
}
