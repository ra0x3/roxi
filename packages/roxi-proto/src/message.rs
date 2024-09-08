use crate::{error::ProtoError, ProtoResult};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

#[repr(u16)]
#[derive(Debug, AsRefStr, Display, Serialize, Deserialize)]
pub enum MessageKind {
    Ping = 0,
    Authenticate = 1,
    Unknown,
}

impl From<u16> for MessageKind {
    fn from(m: u16) -> Self {
        match m {
            0 => MessageKind::Ping,
            1 => MessageKind::Authenticate,
            _ => MessageKind::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    kind: MessageKind,
    data: Option<Vec<u8>>,
}

impl Message {
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

        let mut sizebuff = [0u8; 8];
        sizebuff.copy_from_slice(&data[2..10]);

        let n = usize::from_le_bytes(sizebuff);

        let payload = data[10..n].to_vec();

        Ok(Self {
            kind,
            data: Some(payload),
        })
    }
}
