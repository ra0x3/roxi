use serde::{Deserialize, Serialize};
use std::{net::IpAddr, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Default, Hash, Clone)]
pub struct WireGuardPeer {
    pub public_key: String,
    pub allowed_ips: String,
    pub endpoint: Option<String>,
    pub persistent_keepalive: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub enum ToolType {
    #[serde(rename = "wgquick")]
    WgQuick,

    #[serde(rename = "boringtun")]
    Boringtun,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct WireGuard {
    pub r#type: ToolType,
    pub wgquick: Option<WgQuick>,
    pub boringtun: Option<Boringtun>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct WgQuick {
    pub config: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Boringtun {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
    pub dns: Option<IpAddr>,
    pub port: u16,
    pub peers: Vec<WireGuardPeer>,
}
