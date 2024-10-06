use crate::{command, ProtoError, ProtoResult};
use roxi_lib::types::{
    Boringtun, ToolType, WgQuick, WireGuard, WireGuardPeer as WireGuardConfigPeer,
};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt,
    fs::{self, File},
    io::Write,
    net::IpAddr,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize, Debug, Hash, Default)]
pub enum WireGuardKeyKind {
    Private,
    #[default]
    Public,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
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

#[derive(Debug)]
pub struct WireGuardInterface {
    pub private_key: WireGuardKey,
    pub public_key: WireGuardKey,
    pub address: String,
    pub port: u16,
    pub dns: Option<IpAddr>,
}

impl Serialize for WireGuardInterface {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("WireGuardInterface", 4)?;
        state.serialize_field("PrivateKey", &self.private_key)?;
        state.serialize_field("Address", &self.address)?;
        state.serialize_field("ListenPort", &self.port)?;
        if let Some(dns) = &self.dns {
            state.serialize_field("Dns", dns)?;
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for WireGuardInterface {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct WireGuardInterfaceVisitor;

        impl<'de> Visitor<'de> for WireGuardInterfaceVisitor {
            type Value = WireGuardInterface;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WireGuardInterface")
            }

            fn visit_map<V>(self, mut map: V) -> Result<WireGuardInterface, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut private_key = None;
                let mut address = None;
                let mut port = None;
                let mut dns = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "PrivateKey" => {
                            private_key =
                                Some(WireGuardKey::from_private(map.next_value()?));
                        }
                        "Address" => {
                            address = Some(map.next_value()?);
                        }
                        "ListenPort" => {
                            port = Some(map.next_value()?);
                        }
                        "Dns" => {
                            dns = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let private_key = private_key.unwrap();
                let address = address.unwrap();
                let port = port.unwrap();

                Ok(WireGuardInterface {
                    private_key,
                    public_key: WireGuardKey {
                        key: "mock".to_string(),
                        kind: WireGuardKeyKind::default(),
                    },
                    address,
                    port,
                    dns,
                })
            }
        }

        deserializer.deserialize_struct(
            "WireGuardInterface",
            &["PrivateKey", "Address", "ListenPort", "Dns"],
            WireGuardInterfaceVisitor,
        )
    }
}

#[derive(Debug, Hash, Clone)]
pub struct WireGuardPeer {
    pub public_key: WireGuardKey,
    pub allowed_ips: String,
    pub endpoint: Option<String>,
    pub persistent_keepalive: Option<u16>,
}

impl Serialize for WireGuardPeer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("WireGuardPeer", 4)?;
        state.serialize_field("PublicKey", &self.public_key)?;
        state.serialize_field("AllowedIps", &self.allowed_ips)?;
        if let Some(endpoint) = &self.endpoint {
            state.serialize_field("Endpoint", endpoint)?;
        }
        if let Some(persistent_keepalive) = &self.persistent_keepalive {
            state.serialize_field("PersistentKeepalive", persistent_keepalive)?;
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for WireGuardPeer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WireGuardPeerVisitor;

        impl<'de> de::Visitor<'de> for WireGuardPeerVisitor {
            type Value = WireGuardPeer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct WireGuardPeer")
            }

            fn visit_map<V>(self, mut map: V) -> Result<WireGuardPeer, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut public_key = None;
                let mut allowed_ips = None;
                let mut endpoint = None;
                let mut persistent_keepalive = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "PublicKey" => {
                            public_key =
                                Some(WireGuardKey::from_public(map.next_value()?));
                        }
                        "AllowedIPs" => {
                            allowed_ips = Some(map.next_value()?);
                        }
                        "Endpoint" => {
                            endpoint = Some(map.next_value()?);
                        }
                        "PersistentKeepalive" => {
                            persistent_keepalive = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let public_key = public_key.unwrap();
                let allowed_ips = allowed_ips.unwrap();

                Ok(WireGuardPeer {
                    public_key,
                    allowed_ips,
                    endpoint,
                    persistent_keepalive,
                })
            }
        }

        deserializer.deserialize_struct(
            "WireGuardPeer",
            &["PublicKey", "AllowedIPs", "Endpoint", "PersistentKeepalive"],
            WireGuardPeerVisitor,
        )
    }
}

impl From<WireGuardConfigPeer> for WireGuardPeer {
    fn from(p: WireGuardConfigPeer) -> Self {
        let WireGuardConfigPeer {
            public_key,
            allowed_ips,
            endpoint,
            persistent_keepalive,
            ..
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
    #[serde(rename = "Interface")]
    pub interface: WireGuardInterface,
    #[serde(rename = "Peer")]
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
                    ..
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
    address: Option<String>,
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

    pub fn address(mut self, address: String) -> Self {
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
