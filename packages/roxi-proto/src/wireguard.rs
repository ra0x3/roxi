use crate::{ProtoError, ProtoResult};
use roxi_lib::types::{WireGuard, WireGuardPeer as WireGuardConfigPeer};
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
    private_key: WireGuardKey,
    public_key: WireGuardKey,
    address: IpAddr,
    port: u16,
    dns: Option<IpAddr>,
}

#[derive(Debug, Serialize, Deserialize, Hash, Clone)]
pub struct WireGuardPeer {
    public_key: WireGuardKey,
    allowed_ips: Vec<IpAddr>,
    endpoint: Option<String>,
    persistent_keepalive: Option<u16>,
}

impl WireGuardPeer {
    pub fn new(
        public_key: WireGuardKey,
        allowed_ips: Vec<IpAddr>,
        endpoint: Option<String>,
        persistent_keepalive: Option<u16>,
    ) -> Self {
        Self {
            public_key,
            allowed_ips,
            endpoint,
            persistent_keepalive,
        }
    }

    pub fn public_key(&self) -> WireGuardKey {
        self.public_key.clone()
    }

    pub fn allowed_ips(&self) -> Vec<IpAddr> {
        self.allowed_ips.clone()
    }

    pub fn endpoint(&self) -> Option<String> {
        self.endpoint.clone()
    }

    pub fn persistent_keepalive(&self) -> Option<u16> {
        self.persistent_keepalive
    }
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
        Self {
            public_key: p.public_key().to_string(),
            allowed_ips: p.allowed_ips(),
            endpoint: p.endpoint(),
            persistent_keepalive: p.persistent_keepalive(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WireGuardConfig {
    interface: WireGuardInterface,
    peers: Vec<WireGuardPeer>,
}

impl WireGuardConfig {
    pub fn public_key(&self) -> WireGuardKey {
        self.interface.public_key.clone()
    }

    pub fn private_key(&self) -> WireGuardKey {
        self.interface.private_key.clone()
    }

    pub fn dns(&self) -> Option<IpAddr> {
        self.interface.dns
    }

    pub fn port(&self) -> u16 {
        self.interface.port
    }

    pub fn address(&self) -> IpAddr {
        self.interface.address
    }

    pub fn peers(&self) -> Vec<WireGuardPeer> {
        self.peers.clone()
    }

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

impl From<WireGuard> for WireGuardConfig {
    fn from(w: WireGuard) -> Self {
        Self {
            interface: WireGuardInterface {
                public_key: WireGuardKey::from_public(w.public_key()),
                private_key: WireGuardKey::from_private(w.private_key()),
                dns: w.dns(),
                address: w.address(),
                port: w.port(),
            },
            peers: w
                .peers()
                .iter()
                .map(|p| WireGuardPeer::from(p.to_owned()))
                .collect::<Vec<WireGuardPeer>>(),
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
