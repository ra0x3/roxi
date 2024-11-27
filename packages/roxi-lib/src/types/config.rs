use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default, Hash, Clone)]
pub struct WireGuardConfPeer {
    pub public_key: String,
    pub allowed_ips: String,
    pub endpoint: Option<String>,
    pub persistent_keepalive: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct WireGuardConf {
    pub config: PathBuf,
}
