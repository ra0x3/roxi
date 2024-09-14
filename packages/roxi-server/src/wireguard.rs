use crate::{ServerError, ServerResult};
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
pub struct WireguardInterface {
    #[serde(rename = "PrivateKey")]
    private_key: PrivateKey,
    #[serde(rename = "Address")]
    address: IpAddr,
    #[serde(rename = "ListenPort")]
    port: u16,
    #[serde(rename = "DNS", skip_serializing_if = "Option::is_none")]
    dns: Option<IpAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireguardPeer {
    #[serde(rename = "PublicKey")]
    public_key: PublicKey,
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
pub struct WireguardConfig {
    interface: WireguardInterface,
    peers: Vec<WireguardPeer>,
}

impl TryFrom<&str> for WireguardConfig {
    type Error = toml::de::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let c: WireguardConfig = toml::from_str(s)?;
        Ok(c)
    }
}

impl TryFrom<&PathBuf> for WireguardConfig {
    type Error = ServerError;
    fn try_from(p: &PathBuf) -> ServerResult<Self> {
        let content = fs::read_to_string(p)?;
        let config: WireguardConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

impl TryFrom<&Path> for WireguardConfig {
    type Error = ServerError;
    fn try_from(p: &Path) -> ServerResult<Self> {
        let content = fs::read_to_string(p)?;
        let config: WireguardConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
