use crate::{command, ProtoError, ProtoResult};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::IpAddr,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum WireGuardKeyKind {
    Private,
    Public,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct WireGuardKey {
    key: String,
    kind: WireGuardKeyKind,
}

impl WireGuardKey {
    pub fn from_public(key: String) -> Self {
        Self {
            kind: WireGuardKeyKind::Public,
            key,
        }
    }

    pub fn from_private(key: String) -> Self {
        Self {
            kind: WireGuardKeyKind::Private,
            key,
        }
    }
}

pub struct WireGuardKeyPair {
    #[allow(unused)]
    pubkey: WireGuardKey,
    privkey: WireGuardKey,
}

impl WireGuardKeyPair {
    pub fn new(pubkey: WireGuardKey, privkey: WireGuardKey) -> Self {
        Self { pubkey, privkey }
    }

    pub fn privkey(&self) -> WireGuardKey {
        self.privkey.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WireGuardInterface {
    #[serde(rename = "PrivateKey")]
    private_key: WireGuardKey,
    #[serde(rename = "Address")]
    address: IpAddr,
    #[serde(rename = "ListenPort")]
    port: u16,
    #[serde(rename = "DNS", skip_serializing_if = "Option::is_none")]
    dns: Option<IpAddr>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WireGuardPeer {
    #[serde(rename = "PublicKey", skip_serializing_if = "Option::is_none")]
    public_key: Option<WireGuardKey>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct WireGuardConfig {
    interface: WireGuardInterface,
    peers: Vec<WireGuardPeer>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WireGuardConfigBuilder {
    private_key: Option<WireGuardKey>,
    address: Option<IpAddr>,
    port: Option<u16>,
    dns: Option<IpAddr>,
    peers: Vec<WireGuardPeer>,
}

impl WireGuardConfigBuilder {
    #[allow(unused)]
    pub fn builder() -> Self {
        Self::default()
    }

    #[allow(unused)]
    pub fn private_key(mut self) -> Self {
        let pair = command::wireguard_keypair().expect("Failed to generate WG keypair");
        self.private_key = Some(pair.privkey());
        self
    }

    #[allow(unused)]
    pub fn address(mut self, address: IpAddr) -> Self {
        self.address = Some(address);
        self
    }

    #[allow(unused)]
    pub fn listen_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    #[allow(unused)]
    pub fn dns(mut self, dns: IpAddr) -> Self {
        self.dns = Some(dns);
        self
    }

    #[allow(unused)]
    pub fn peer(mut self, peer: WireGuardPeer) -> Self {
        self.peers.push(peer);
        self
    }

    #[allow(unused)]
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
