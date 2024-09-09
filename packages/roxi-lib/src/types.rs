use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum StunAddressKind {
    Public = 0,
    Private = 1,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct StunInfo {
    kind: StunAddressKind,
    ip: Ipv4Addr,
    port: u16,
}

impl StunInfo {
    pub fn new(kind: StunAddressKind, ip: Ipv4Addr, port: u16) -> Self {
        Self { kind, ip, port }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct ClientId(String);

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ClientId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ClientId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl TryFrom<&TcpStream> for ClientId {
    type Error = std::io::Error;
    fn try_from(s: &TcpStream) -> Result<Self, Self::Error> {
        let addr = s.peer_addr()?;
        Ok(Self::from(addr.ip().to_string()))
    }
}

impl From<&SocketAddr> for ClientId {
    fn from(s: &SocketAddr) -> Self {
        Self::from(s.ip().to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Default)]
pub struct SharedKey(String);

impl SharedKey {
    pub fn to_vec(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}

impl std::fmt::Display for SharedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "roxi-XXX")
    }
}

impl From<&str> for SharedKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl TryFrom<SharedKey> for Vec<u8> {
    type Error = std::string::FromUtf8Error;
    fn try_from(k: SharedKey) -> Result<Self, Self::Error> {
        Ok(k.to_vec())
    }
}

impl TryFrom<Vec<u8>> for SharedKey {
    type Error = std::string::FromUtf8Error;
    fn try_from(k: Vec<u8>) -> Result<Self, Self::Error> {
        let res = String::from_utf8(k)?;
        Ok(Self(res))
    }
}
