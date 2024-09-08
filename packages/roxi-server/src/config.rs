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
    max_clients: usize,
}

impl Client {
    pub fn max_clients(&self) -> usize {
        self.max_clients
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

impl Auth {
    pub fn shared_key(&self) -> SharedKey {
        self.shared_key.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    interface: Ipv4Addr,
    port: u16,
}

impl Server {
    pub fn hostname(&self) -> String {
        format!("{}:{}", self.interface, self.port)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    server: Server,
    client: Client,
    tun: Tun,
    auth: Auth,
}

impl Config {
    pub fn hostname(&self) -> String {
        self.server.hostname()
    }

    pub fn max_clients(&self) -> usize {
        self.client.max_clients()
    }

    pub fn shared_key(&self) -> SharedKey {
        self.auth.shared_key()
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
