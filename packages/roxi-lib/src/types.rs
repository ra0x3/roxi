use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

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

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct SharedKey(String);

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

impl Into<Vec<u8>> for SharedKey {
    fn into(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}

impl TryFrom<Vec<u8>> for SharedKey {
    type Error = std::string::FromUtf8Error;
    fn try_from(k: Vec<u8>) -> Result<Self, Self::Error> {
        let res = String::from_utf8(k)?;
        Ok(Self(res))
    }
}
