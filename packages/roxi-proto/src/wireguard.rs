use crate::{command, ProtoError, ProtoResult};
use roxi_lib::types::{
    Boringtun, ToolType, WgQuick, WireGuard, WireGuardPeer as WireGuardConfigPeer,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fs::{self, File},
    io::Write,
    net::IpAddr,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub enum WireGuardKeyKind {
    Private,
    Public,
}

#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
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

    pub fn as_bytes(&mut self) -> &[u8] {
        self.key.as_bytes()
    }
}

impl fmt::Display for WireGuardKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl TryFrom<&PathBuf> for WireGuardKey {
    type Error = ProtoError;
    fn try_from(p: &PathBuf) -> ProtoResult<Self> {
        let k = command::cat_wireguard_key(p)?;
        Ok(k)
    }
}

pub struct WireGuardKeyPair {
    #[allow(unused)]
    pub pubkey: WireGuardKey,
    pub privkey: WireGuardKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WireGuardInterface {
    pub private_key: WireGuardKey,
    pub public_key: WireGuardKey,
    pub address: IpAddr,
    pub port: u16,
    pub dns: Option<IpAddr>,
}

#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
pub struct WireGuardPeer {
    pub public_key: WireGuardKey,
    pub allowed_ips: Vec<IpAddr>,
    pub endpoint: Option<String>,
    pub persistent_keepalive: Option<u16>,
}

impl From<WireGuardConfigPeer> for WireGuardPeer {
    fn from(p: WireGuardConfigPeer) -> Self {
        let WireGuardConfigPeer {
            public_key,
            allowed_ips,
            endpoint,
            persistent_keepalive,
        } = p;
        Self {
            public_key: WireGuardKey::from_public(public_key),
            allowed_ips,
            endpoint,
            persistent_keepalive,
        }
    }
}

impl From<&WireGuardPeer> for WireGuardConfigPeer {
    fn from(p: &WireGuardPeer) -> Self {
        let WireGuardPeer {
            public_key,
            allowed_ips,
            endpoint,
            persistent_keepalive,
        } = p.to_owned();
        Self {
            public_key: public_key.to_string(),
            allowed_ips,
            endpoint,
            persistent_keepalive,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub interface: WireGuardInterface,
    pub peers: Vec<WireGuardPeer>,
}

impl WireGuardConfig {
    pub fn add_peer(&mut self, p: WireGuardPeer) {
        self.peers.push(p)
    }

    pub fn save<P: AsRef<Path>>(&self, p: P) -> ProtoResult<()> {
        let content = toml::to_string(&self)?;
        let mut f = File::create(&p)?;
        f.write_all(content.as_bytes())?;
        Ok(())
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

impl TryFrom<WireGuard> for WireGuardConfig {
    type Error = ProtoError;
    fn try_from(w: WireGuard) -> Result<Self, Self::Error> {
        let WireGuard {
            r#type,
            boringtun,
            wgquick,
        } = w;
        match r#type {
            ToolType::WgQuick => {
                if let Some(WgQuick { config }) = wgquick {
                    let config = WireGuardConfig::try_from(&config)?;
                    Ok(config)
                } else {
                    Err(ProtoError::MalformedConfig)
                }
            }
            ToolType::Boringtun => {
                if let Some(Boringtun {
                    public_key,
                    private_key,
                    dns,
                    address,
                    port,
                    peers,
                }) = boringtun
                {
                    Ok(Self {
                        interface: WireGuardInterface {
                            public_key: WireGuardKey::from_public(public_key),
                            private_key: WireGuardKey::from_private(private_key),
                            dns,
                            address,
                            port,
                        },
                        peers: peers
                            .iter()
                            .map(|p| WireGuardPeer::from(p.to_owned()))
                            .collect::<Vec<WireGuardPeer>>(),
                    })
                } else {
                    Err(ProtoError::MalformedConfig)
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WireGuardConfigBuilder {
    private_key: Option<WireGuardKey>,
    public_key: Option<WireGuardKey>,
    address: Option<IpAddr>,
    port: Option<u16>,
    dns: Option<IpAddr>,
    peers: Vec<WireGuardPeer>,
}

impl WireGuardConfigBuilder {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn private_key(mut self, k: String) -> Self {
        let key = WireGuardKey::from_private(k);
        self.private_key = Some(key);
        self
    }

    pub fn public_key(mut self, k: String) -> Self {
        let key = WireGuardKey::from_public(k);
        self.public_key = Some(key);
        self
    }

    pub fn address(mut self, address: IpAddr) -> Self {
        self.address = Some(address);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn dns(mut self, dns: Option<IpAddr>) -> Self {
        self.dns = dns;
        self
    }

    pub fn peer(mut self, peer: WireGuardPeer) -> Self {
        self.peers.push(peer);
        self
    }

    pub fn peers(mut self, peers: Vec<WireGuardPeer>) -> Self {
        self.peers = peers;
        self
    }

    pub fn build(self) -> WireGuardConfig {
        WireGuardConfig {
            interface: WireGuardInterface {
                private_key: self.private_key.expect("Private key expected"),
                public_key: self.public_key.expect("Public key expected"),
                address: self.address.expect("Address expected"),
                port: self.port.expect("Port expected"),
                dns: self.dns,
            },
            peers: self.peers,
        }
    }
}
