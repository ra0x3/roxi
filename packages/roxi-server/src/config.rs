use crate::{error::ServerError, ServerResult};
use roxi_lib::types::SharedKey;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Client {
    limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IP {
    pool: Vec<Ipv4Addr>,
}

impl IP {
    pub fn pool(&self) -> Vec<Ipv4Addr> {
        self.pool.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tun {
    address: Ipv4Addr,
    netmask: Ipv4Addr,
    name: String,
    destination: Ipv4Addr,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    expiry: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Auth {
    shared_key: SharedKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    host: String,
    port: String,
    client: Client,
    ip: IP,
    tun: Tun,
    auth: Auth,
}

impl Config {
    pub fn hostname(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn ip_pool(&self) -> Vec<Ipv4Addr> {
        self.ip.pool()
    }
}

impl TryFrom<&PathBuf> for Config {
    type Error = ServerError;
    fn try_from(p: &PathBuf) -> ServerResult<Self> {
        let file = File::open(p)?;
        let content: serde_yaml::Value = serde_yaml::from_reader(file)?;
        let config: Config = serde_yaml::from_value(content)?;
        Ok(config)
    }
}

impl TryFrom<&Path> for Config {
    type Error = ServerError;
    fn try_from(p: &Path) -> ServerResult<Self> {
        let file = File::open(p)?;
        let content: serde_yaml::Value = serde_yaml::from_reader(file)?;
        let config: Config = serde_yaml::from_value(content)?;
        Ok(config)
    }
}

impl TryFrom<&str> for Config {
    type Error = ServerError;
    fn try_from(s: &str) -> ServerResult<Self> {
        let content: serde_yaml::Value = serde_yaml::from_str(s)?;
        let config: Config = serde_yaml::from_value(content)?;
        Ok(config)
    }
}
