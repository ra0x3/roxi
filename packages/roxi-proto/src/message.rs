use crate::{error::ProtoError, ProtoResult};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use strum::{AsRefStr, Display};

#[repr(u16)]
#[derive(Debug, AsRefStr, Display, Serialize, Deserialize)]
pub enum MessageKind {
    Ping = 0,
    Pong = 1,
    AuthenticationRequest = 2,
    AuthenticationResponse = 3,
    Unknown,
}

impl From<u16> for MessageKind {
    fn from(m: u16) -> Self {
        match m {
            0 => MessageKind::Ping,
            1 => MessageKind::Pong,
            2 => MessageKind::AuthenticationRequest,
            3 => MessageKind::AuthenticationResponse,
            _ => MessageKind::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    kind: MessageKind,
    hostname: [u8; 6],
    data: Option<Vec<u8>>,
}

impl Message {
    pub fn new(kind: MessageKind, hostname: String, data: Option<Vec<u8>>) -> Self {
        let hostname = Message::hostname_to_arr(hostname);
        Self {
            kind,
            hostname,
            data,
        }
    }

    fn hostname_to_arr(hostname: String) -> [u8; 6] {
        let mut parts = hostname.split(':');
        let ip: Ipv4Addr = parts
            .next()
            .expect("Invalid hostname ip")
            .parse()
            .expect("Bad address ip");
        let mut ip = ip.octets();
        ip.reverse();

        let port = parts
            .next()
            .expect("Invalid hostname port")
            .parse::<u16>()
            .expect("Bad address port")
            .to_le_bytes();

        let mut buff = [0u8; 6];
        buff[..4].copy_from_slice(&ip);
        buff[4..].copy_from_slice(&port);

        buff
    }

    pub fn into_inner(&mut self) -> Option<Vec<u8>> {
        self.data.take()
    }

    pub fn kind(&self) -> &MessageKind {
        &self.kind
    }

    pub fn serialize(self) -> ProtoResult<Vec<u8>> {
        let mut result = Vec::new();
        let data = self.data.unwrap_or_else(Vec::new);
        result.extend(&(self.kind as u16).to_le_bytes());
        result.extend(&self.hostname);
        let len = data.len() as usize;
        result.extend(&len.to_le_bytes());
        result.extend(&data);

        Ok(result)
    }

    pub fn deserialize(data: &[u8]) -> ProtoResult<Self> {
        if data.len() < 10 {
            return Err(ProtoError::MalformedMessage);
        }

        let mut kindbuff = [0u8; 2];
        kindbuff.copy_from_slice(&data[..2]);
        let kind: MessageKind = u16::from_le_bytes(kindbuff).try_into().unwrap();

        let mut hostnamebuff = [0u8; 6];
        hostnamebuff.copy_from_slice(&data[2..8]);

        let mut sizebuff = [0u8; 8];
        sizebuff.copy_from_slice(&data[8..16]);

        let payload = match kind {
            MessageKind::Ping | MessageKind::Pong => None,
            _ => {
                let n = usize::from_le_bytes(sizebuff);
                let payload = data[16..n].to_vec();
                Some(payload)
            }
        };

        Ok(Self {
            kind,
            hostname: hostnamebuff,
            data: payload,
        })
    }
}
