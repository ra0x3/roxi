use crate::{error::ProtoError, ProtoResult};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use strum::{AsRefStr, Display};

#[repr(u16)]
#[derive(Debug, AsRefStr, Display, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending = 0,
    r#Ok = 200,
    Created = 201,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    BadData = 405,
    ImATeapot = 419,
    InternalServerError = 500,
    Unknown,
}

impl From<u16> for MessageStatus {
    fn from(m: u16) -> Self {
        match m {
            0 => MessageStatus::Pending,
            200 => MessageStatus::r#Ok,
            201 => MessageStatus::Created,
            401 => MessageStatus::Unauthorized,
            403 => MessageStatus::Forbidden,
            404 => MessageStatus::NotFound,
            405 => MessageStatus::BadData,
            419 => MessageStatus::ImATeapot,
            500 => MessageStatus::InternalServerError,
            _ => MessageStatus::Unknown,
        }
    }
}

#[repr(u16)]
#[derive(Debug, AsRefStr, Display, Serialize, Deserialize)]
pub enum MessageKind {
    Ping = 0,
    Pong = 1,
    AuthenticationRequest = 2,
    AuthenticationResponse = 3,
    StunRequest = 4,
    StunResponse = 5,
    DisconnectSessionRequest = 6,
    DisconnectSessionResponse = 7,
    StunInfoRequest = 8,
    StunInfoResponse = 9,
    GatewayRequest = 10,
    GatewayResponse = 11,
    GenericErrorResponse = 12,
    PeerTunnelRequest = 13,
    PeerTunnelResponse = 14,
    NATPunchRequest = 15,
    NATPunchResponse = 16,
    PeerTunnelInitRequest = 17,
    PeerTunnelInitResponse = 18,
    Unknown,
}

impl From<u16> for MessageKind {
    fn from(m: u16) -> Self {
        match m {
            0 => MessageKind::Ping,
            1 => MessageKind::Pong,
            2 => MessageKind::AuthenticationRequest,
            3 => MessageKind::AuthenticationResponse,
            4 => MessageKind::StunRequest,
            5 => MessageKind::StunResponse,
            6 => MessageKind::DisconnectSessionRequest,
            7 => MessageKind::DisconnectSessionResponse,
            8 => MessageKind::StunInfoRequest,
            9 => MessageKind::StunInfoResponse,
            10 => MessageKind::GatewayRequest,
            11 => MessageKind::GatewayResponse,
            12 => MessageKind::GenericErrorResponse,
            13 => MessageKind::PeerTunnelRequest,
            14 => MessageKind::PeerTunnelResponse,
            15 => MessageKind::NATPunchRequest,
            16 => MessageKind::NATPunchResponse,
            17 => MessageKind::PeerTunnelInitRequest,
            18 => MessageKind::PeerTunnelInitResponse,
            _ => MessageKind::Unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    kind: MessageKind,
    status: MessageStatus,
    sender_addr: [u8; 6],
    data: Option<Vec<u8>>,
}

impl Message {
    pub fn new(
        kind: MessageKind,
        status: MessageStatus,
        addr: String,
        data: Option<Vec<u8>>,
    ) -> Self {
        let sender_addr = Message::pack_addr(addr);
        Self {
            kind,
            status,
            sender_addr,
            data,
        }
    }

    fn pack_addr(hostname: String) -> [u8; 6] {
        let mut parts = hostname.split(':');
        let ip: Ipv4Addr = parts
            .next()
            .expect("Invalid hostname ip")
            .parse()
            .expect("Bad address ip");
        let ip = ip.octets();

        let port = parts
            .next()
            .expect("Invalid hostname port")
            .parse::<u16>()
            .expect("Bad address port")
            .to_be_bytes();

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

    pub fn status(&self) -> &MessageStatus {
        &self.status
    }

    pub fn sender_addr(&self) -> [u8; 6] {
        self.sender_addr
    }

    pub fn data(&self) -> Vec<u8> {
        self.data.clone().unwrap_or_default()
    }

    pub fn serialize(self) -> ProtoResult<Vec<u8>> {
        let mut result = Vec::new();
        let data = self.data.unwrap_or_default();
        result.extend(&(self.kind as u16).to_be_bytes());
        result.extend(&(self.status as u16).to_be_bytes());
        result.extend(&self.sender_addr);
        result.extend(&(data.len().to_be_bytes()));
        result.extend(&data);

        Ok(result)
    }

    pub fn deserialize(data: &[u8]) -> ProtoResult<Self> {
        if data.len() < 10 {
            return Err(ProtoError::MalformedMessage);
        }

        let mut kindbuff = [0u8; 2];
        kindbuff.copy_from_slice(&data[..2]);
        let kind: MessageKind = u16::from_be_bytes(kindbuff).into();

        let mut statusbuff = [0u8; 2];
        statusbuff.copy_from_slice(&data[2..4]);
        let status: MessageStatus = u16::from_be_bytes(statusbuff).into();

        let mut addrbuff = [0u8; 6];
        addrbuff.copy_from_slice(&data[4..10]);

        let mut sizebuff = [0u8; 8];
        sizebuff.copy_from_slice(&data[10..18]);

        let payload = match kind {
            MessageKind::Ping
            | MessageKind::Pong
            | MessageKind::StunInfoRequest
            | MessageKind::AuthenticationResponse => None,
            _ => {
                let n = usize::from_be_bytes(sizebuff);
                let payload = data[18..18 + n].to_vec();
                Some(payload)
            }
        };

        Ok(Self {
            kind,
            status,
            sender_addr: addrbuff,
            data: payload,
        })
    }
}
