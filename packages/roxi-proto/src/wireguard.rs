use crate::{ProtoError, ProtoResult};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::IpAddr,
    path::{Path, PathBuf},
};

type Key = [u8; 32];
pub type PublicKey = Key;
pub type PrivateKey = Key;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardInterface {
    #[serde(rename = "PrivateKey")]
    private_key: PrivateKey,
    #[serde(rename = "Address")]
    address: IpAddr,
    #[serde(rename = "ListenPort")]
    port: u16,
    #[serde(rename = "DNS", skip_serializing_if = "Option::is_none")]
    dns: Option<IpAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WireGuardPeer {
    #[serde(rename = "PublicKey", skip_serializing_if = "Option::is_none")]
    public_key: Option<PublicKey>,
    #[serde(rename = "AllowedIPs")]
    allowed_ips: Vec<IpAddr>,
    #[serde(rename = "Endpoint", skip_serializing_if = "Option::is_none")]
    endpoint: Option<String>,
    #[serde(
        rename = "PersistentKeepalive",
        skip_serializing_if = "Option::is_none"
    )]
    persistent_keepalive: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardConfig {
    interface: WireGuardInterface,
    peers: Vec<WireGuardPeer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WireGuardConfigBuilder {
    private_key: Option<[u8; 32]>,
    address: Option<IpAddr>,
    port: Option<u16>,
    dns: Option<IpAddr>,
    peers: Vec<WireGuardPeer>,
}

impl WireGuardConfigBuilder {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn private_key(mut self, k: &str) -> Self {
        let mut key = [0u8; 32];
        key.copy_from_slice(&k.as_bytes()[..32]);
        self.private_key = Some(key);
        self
    }

    pub fn address(mut self, address: IpAddr) -> Self {
        self.address = Some(address);
        self
    }

    pub fn listen_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn dns(mut self, dns: IpAddr) -> Self {
        self.dns = Some(dns);
        self
    }

    pub fn peer(mut self, peer: WireGuardPeer) -> Self {
        self.peers.push(peer);
        self
    }

    pub fn build(self) -> WireGuardConfig {
        WireGuardConfig {
            interface: WireGuardInterface {
                private_key: self.private_key.expect("Private key expected"),
                address: self.address.expect("Address expected"),
                port: self.port.expect("Port expected"),
                dns: self.dns,
            },
            peers: self.peers,
        }
    }
}

impl TryFrom<&str> for WireGuardConfig {
    type Error = toml::de::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let c: WireGuardConfig = toml::from_str(s)?;
        Ok(c)
    }
}

impl TryFrom<&PathBuf> for WireGuardConfig {
    type Error = ProtoError;
    fn try_from(p: &PathBuf) -> ProtoResult<Self> {
        let content = fs::read_to_string(p)?;
        let config: WireGuardConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

impl TryFrom<&Path> for WireGuardConfig {
    type Error = ProtoError;
    fn try_from(p: &Path) -> ProtoResult<Self> {
        let content = fs::read_to_string(p)?;
        let config: WireGuardConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
