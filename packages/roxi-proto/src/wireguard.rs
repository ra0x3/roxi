use crate::{command, ProtoError, ProtoResult};
use roxi_lib::types::config::{self};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt,
    fs::{self, File},
    io::Write,
    net::IpAddr,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize, Debug, Hash, Default)]
pub enum WireGuardProtoKeyKind {
    Private,
    #[default]
    Public,
}

#[derive(Clone, Debug, Hash, Deserialize)]
pub struct WireGuardProtoKey {
    key: String,
    kind: WireGuardProtoKeyKind,
}

impl WireGuardProtoKey {
    pub fn from_public(key: String) -> Self {
        Self {
            kind: WireGuardProtoKeyKind::Public,
            key,
        }
    }

    pub fn from_private(key: String) -> Self {
        Self {
            kind: WireGuardProtoKeyKind::Private,
            key,
        }
    }

    pub fn as_bytes(&mut self) -> &[u8] {
        self.key.as_bytes()
    }
}

impl fmt::Display for WireGuardProtoKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

impl TryFrom<&PathBuf> for WireGuardProtoKey {
    type Error = ProtoError;
    fn try_from(p: &PathBuf) -> ProtoResult<Self> {
        let k = command::cat_wireguard_key(p)?;
        Ok(k)
    }
}

impl Serialize for WireGuardProtoKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.key)
    }
}

pub struct WireGuardProtoKeyPair {
    #[allow(unused)]
    pub pubkey: WireGuardProtoKey,
    pub privkey: WireGuardProtoKey,
}

#[derive(Debug)]
pub struct WireGuardProtoInterface {
    pub private_key: WireGuardProtoKey,
    pub address: String,
    pub port: u16,
    pub dns: Option<IpAddr>,
}

impl Serialize for WireGuardProtoInterface {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("WireGuardProtoInterface", 4)?;
        state.serialize_field("PrivateKey", &self.private_key)?;
        state.serialize_field("Address", &self.address)?;
        state.serialize_field("ListenPort", &self.port)?;
        if let Some(dns) = &self.dns {
            state.serialize_field("Dns", dns)?;
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for WireGuardProtoInterface {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct WireGuardProtoInterfaceVisitor;

        impl<'de> Visitor<'de> for WireGuardProtoInterfaceVisitor {
            type Value = WireGuardProtoInterface;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WireGuardProtoInterface")
            }

            fn visit_map<V>(self, mut map: V) -> Result<WireGuardProtoInterface, V::Error>
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
                                Some(WireGuardProtoKey::from_private(map.next_value()?));
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

                Ok(WireGuardProtoInterface {
                    private_key,
                    address,
                    port,
                    dns,
                })
            }
        }

        deserializer.deserialize_struct(
            "WireGuardProtoInterface",
            &["PrivateKey", "Address", "ListenPort", "Dns"],
            WireGuardProtoInterfaceVisitor,
        )
    }
}

#[derive(Debug, Hash, Clone)]
pub struct WireGuardProtoPeer {
    pub public_key: WireGuardProtoKey,
    pub allowed_ips: String,
    pub endpoint: Option<String>,
    pub persistent_keepalive: Option<u16>,
}

impl Serialize for WireGuardProtoPeer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("WireGuardProtoPeer", 4)?;
        state.serialize_field("PublicKey", &self.public_key)?;
        state.serialize_field("AllowedIPs", &self.allowed_ips)?;
        if let Some(endpoint) = &self.endpoint {
            state.serialize_field("Endpoint", endpoint)?;
        }
        if let Some(persistent_keepalive) = &self.persistent_keepalive {
            state.serialize_field("PersistentKeepalive", persistent_keepalive)?;
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for WireGuardProtoPeer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WireGuardProtoPeerVisitor;

        impl<'de> de::Visitor<'de> for WireGuardProtoPeerVisitor {
            type Value = WireGuardProtoPeer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct WireGuardProtoPeer")
            }

            fn visit_map<V>(self, mut map: V) -> Result<WireGuardProtoPeer, V::Error>
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
                                Some(WireGuardProtoKey::from_public(map.next_value()?));
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

                Ok(WireGuardProtoPeer {
                    public_key,
                    allowed_ips,
                    endpoint,
                    persistent_keepalive,
                })
            }
        }

        deserializer.deserialize_struct(
            "WireGuardProtoPeer",
            &["PublicKey", "AllowedIPs", "Endpoint", "PersistentKeepalive"],
            WireGuardProtoPeerVisitor,
        )
    }
}

impl From<config::WireGuardConfPeer> for WireGuardProtoPeer {
    fn from(p: config::WireGuardConfPeer) -> Self {
        let config::WireGuardConfPeer {
            public_key,
            allowed_ips,
            endpoint,
            persistent_keepalive,
            ..
        } = p;
        Self {
            public_key: WireGuardProtoKey::from_public(public_key),
            allowed_ips,
            endpoint,
            persistent_keepalive,
        }
    }
}

impl From<&WireGuardProtoPeer> for config::WireGuardConfPeer {
    fn from(p: &WireGuardProtoPeer) -> Self {
        let WireGuardProtoPeer {
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

#[derive(Debug, Serialize)]
pub struct WireGuardProtoConfig {
    #[serde(rename = "Interface")]
    pub interface: WireGuardProtoInterface,
    #[serde(rename = "Peer", default)]
    pub peers: Option<Vec<WireGuardProtoPeer>>,
}

impl WireGuardProtoConfig {
    pub fn add_peer(&mut self, p: WireGuardProtoPeer) {
        match self.peers {
            Some(ref mut peers) => {
                peers.push(p);
            }
            None => {
                self.peers = Some(vec![p]);
            }
        }
    }

    pub fn save<P: AsRef<Path>>(&self, p: P) -> ProtoResult<()> {
        let content = toml::to_string(&self)?;
        let mut f = File::create(&p)?;
        f.write_all(content.as_bytes())?;
        Ok(())
    }
}

impl TryFrom<&str> for WireGuardProtoConfig {
    type Error = toml::de::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let c: WireGuardProtoConfig = toml::from_str(s)?;
        Ok(c)
    }
}

impl TryFrom<&PathBuf> for WireGuardProtoConfig {
    type Error = ProtoError;
    fn try_from(p: &PathBuf) -> ProtoResult<Self> {
        let content = fs::read_to_string(p)?;
        let config: WireGuardProtoConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

impl TryFrom<&Path> for WireGuardProtoConfig {
    type Error = ProtoError;
    fn try_from(p: &Path) -> ProtoResult<Self> {
        let content = fs::read_to_string(p)?;
        let config: WireGuardProtoConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

impl TryFrom<config::WireGuardConf> for WireGuardProtoConfig {
    type Error = ProtoError;
    fn try_from(w: config::WireGuardConf) -> Result<Self, Self::Error> {
        let config::WireGuardConf { config } = w;
        let config = WireGuardProtoConfig::try_from(&config)?;
        Ok(config)
    }
}

impl<'de> Deserialize<'de> for WireGuardProtoConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WireGuardProtoConfigVisitor;

        impl<'de> de::Visitor<'de> for WireGuardProtoConfigVisitor {
            type Value = WireGuardProtoConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WireGuardProtoConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<WireGuardProtoConfig, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut interface = None;
                let mut peers = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "Interface" => {
                            interface = Some(map.next_value()?);
                        }
                        "Peer" => {
                            peers = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(WireGuardProtoConfig {
                    interface: interface
                        .ok_or_else(|| de::Error::missing_field("Interface"))?,
                    peers,
                })
            }
        }

        deserializer.deserialize_struct(
            "WireGuardProtoConfig",
            &["Interface", "Peer"],
            WireGuardProtoConfigVisitor,
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WireGuardProtoConfigBuilder {
    private_key: Option<WireGuardProtoKey>,
    public_key: Option<WireGuardProtoKey>,
    address: Option<String>,
    port: Option<u16>,
    dns: Option<IpAddr>,
    peers: Option<Vec<WireGuardProtoPeer>>,
}

impl WireGuardProtoConfigBuilder {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn private_key(mut self, k: String) -> Self {
        let key = WireGuardProtoKey::from_private(k);
        self.private_key = Some(key);
        self
    }

    pub fn public_key(mut self, k: String) -> Self {
        let key = WireGuardProtoKey::from_public(k);
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

    pub fn peer(mut self, peer: WireGuardProtoPeer) -> Self {
        match self.peers {
            Some(ref mut p) => {
                p.push(peer);
            }
            None => {
                vec![peer];
            }
        }
        self
    }

    pub fn peers(mut self, peers: Vec<WireGuardProtoPeer>) -> Self {
        self.peers = Some(peers);
        self
    }

    pub fn build(self) -> WireGuardProtoConfig {
        WireGuardProtoConfig {
            interface: WireGuardProtoInterface {
                private_key: self.private_key.expect("Private key expected"),
                address: self.address.expect("Address expected"),
                port: self.port.expect("Port expected"),
                dns: self.dns,
            },
            peers: self.peers,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_wireguard_config_can_be_updated_and_saved() {
        let content = r#"
[Interface]
# The private key of the WireGuard server (keep this secret)
PrivateKey = "ServerPrivateKey"

# The IP address and subnet of the WireGuard interface on the server
Address = "10.0.0.1/24"

# The UDP port on which WireGuard will listen
ListenPort = 51820

# The location where persistent data (peers) will be stored
# Optional, but useful for storing peer data across reboots
SaveConfig = true

# Optional: Firewall settings to allow traffic
# PostUp and PostDown run commands after the interface is brought up or down
# PostUp = iptables -A FORWARD -i wg0 -j ACCEPT; iptables -A FORWARD -o wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
# PostDown = iptables -D FORWARD -i wg0 -j ACCEPT; iptables -D FORWARD -o wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE
"#;

        let mut config = WireGuardProtoConfig::try_from(content).unwrap();
        config.add_peer(WireGuardProtoPeer {
            public_key: WireGuardProtoKey::from_public("123".to_string()),
            allowed_ips: "".to_string(),
            endpoint: None,
            persistent_keepalive: None,
        });

        let name = "wg0.test.conf.foo";
        let _ = config.save(name);

        let path = PathBuf::from(name);
        let updated_config = WireGuardProtoConfig::try_from(&path).unwrap();
        assert_eq!(
            updated_config.peers.as_ref().map(|v| v.len()).unwrap_or(0),
            1
        );

        config.add_peer(WireGuardProtoPeer {
            public_key: WireGuardProtoKey::from_public("456".to_string()),
            allowed_ips: "".to_string(),
            endpoint: None,
            persistent_keepalive: None,
        });

        config.add_peer(WireGuardProtoPeer {
            public_key: WireGuardProtoKey::from_public("789".to_string()),
            allowed_ips: "".to_string(),
            endpoint: None,
            persistent_keepalive: None,
        });

        let _ = config.save(name);

        let updated_config = WireGuardProtoConfig::try_from(&path).unwrap();
        assert_eq!(
            updated_config.peers.as_ref().map(|v| v.len()).unwrap_or(0),
            3
        );

        std::fs::remove_file(name).unwrap();
    }
}
