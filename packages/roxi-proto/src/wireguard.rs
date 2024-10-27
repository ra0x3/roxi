use crate::{command, ProtoError, ProtoResult};
use roxi_lib::types::config::{self, Boringtun, ToolType, WgQuick};
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

#[derive(Clone, Debug, Hash, Deserialize)]
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

impl Serialize for WireGuardKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.key)
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

impl From<config::WireGuardPeer> for WireGuardPeer {
    fn from(p: config::WireGuardPeer) -> Self {
        let config::WireGuardPeer {
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

impl From<&WireGuardPeer> for config::WireGuardPeer {
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

#[derive(Debug, Serialize)]
pub struct WireGuardConfig {
    #[serde(rename = "Interface")]
    pub interface: WireGuardInterface,
    #[serde(rename = "Peer", default)]
    pub peers: Option<Vec<WireGuardPeer>>,
}

impl WireGuardConfig {
    pub fn add_peer(&mut self, p: WireGuardPeer) {
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

impl TryFrom<config::WireGuard> for WireGuardConfig {
    type Error = ProtoError;
    fn try_from(w: config::WireGuard) -> Result<Self, Self::Error> {
        let config::WireGuard {
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
                            private_key: WireGuardKey::from_private(private_key),
                            dns,
                            address,
                            port,
                        },
                        peers: Some(
                            peers
                                .iter()
                                .map(|p| WireGuardPeer::from(p.to_owned()))
                                .collect::<Vec<WireGuardPeer>>(),
                        ),
                    })
                } else {
                    Err(ProtoError::MalformedConfig)
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for WireGuardConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WireGuardConfigVisitor;

        impl<'de> de::Visitor<'de> for WireGuardConfigVisitor {
            type Value = WireGuardConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WireGuardConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<WireGuardConfig, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut interface = None;
                let mut peers = None;

                while let Some(key) = map.next_key::<String>()? {
                    println!("DEBUG: Found key in config: {}", key);  // Debug print
                    match key.as_str() {
                        "Interface" => {
                            interface = Some(map.next_value()?);
                        }
                        "Peer" => {
                            peers = Some(map.next_value()?);
                        }
                        _ => {
                            println!("DEBUG: Ignoring unknown key: {}", key);  // Debug print
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(WireGuardConfig {
                    interface: interface.ok_or_else(|| de::Error::missing_field("Interface"))?,
                    peers,
                })
            }
        }

        deserializer.deserialize_struct(
            "WireGuardConfig",
            &["Interface", "Peer"],
            WireGuardConfigVisitor
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WireGuardConfigBuilder {
    private_key: Option<WireGuardKey>,
    public_key: Option<WireGuardKey>,
    address: Option<String>,
    port: Option<u16>,
    dns: Option<IpAddr>,
    peers: Option<Vec<WireGuardPeer>>,
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

    pub fn peers(mut self, peers: Vec<WireGuardPeer>) -> Self {
        self.peers = Some(peers);
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

        let mut config = WireGuardConfig::try_from(content).unwrap();
        config.add_peer(WireGuardPeer {
            public_key: WireGuardKey::from_public("123".to_string()),
            allowed_ips: "".to_string(),
            endpoint: None,
            persistent_keepalive: None,
        });

        let name = "wg0.test.conf.foo";
        let _ = config.save(name);

        let path = PathBuf::from(name);
        let updated_config = WireGuardConfig::try_from(&path).unwrap();
        assert_eq!(
            updated_config.peers.as_ref().map(|v| v.len()).unwrap_or(0),
            1
        );

        config.add_peer(WireGuardPeer {
            public_key: WireGuardKey::from_public("456".to_string()),
            allowed_ips: "".to_string(),
            endpoint: None,
            persistent_keepalive: None,
        });

        config.add_peer(WireGuardPeer {
            public_key: WireGuardKey::from_public("789".to_string()),
            allowed_ips: "".to_string(),
            endpoint: None,
            persistent_keepalive: None,
        });

        let _ = config.save(name);

        let updated_config = WireGuardConfig::try_from(&path).unwrap();
        assert_eq!(
            updated_config.peers.as_ref().map(|v| v.len()).unwrap_or(0),
            3
        );

        std::fs::remove_file(name).unwrap();
    }
}
