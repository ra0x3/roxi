use crate::{error::ServerError, ServerResult};
use roxi_lib::types::SharedKey;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tun {
    address: Ipv4Addr,
    netmask: Ipv4Addr,
    name: String,
    destination: Ipv4Addr,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Auth {
    shared_key: SharedKey,
    session_ttl: u32,
}

impl Auth {
    pub fn shared_key(&self) -> SharedKey {
        self.shared_key.clone()
    }

    pub fn session_ttl(&self) -> u32 {
        self.session_ttl
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    interface: Ipv4Addr,
    ip: Ipv4Addr,
    port: u16,
    max_clients: u32,
}

impl Server {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn interface(&self) -> String {
        format!("{}:{}", self.interface, self.port)
    }

    pub fn max_clients(&self) -> u32 {
        self.max_clients
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    server: Server,
    tun: Tun,
    auth: Auth,
    path: String,
}

impl Config {
    pub fn interface(&self) -> String {
        self.server.interface()
    }

    pub fn addr(&self) -> String {
        self.server.addr()
    }

    pub fn max_clients(&self) -> u32 {
        self.server.max_clients()
    }

    pub fn shared_key(&self) -> SharedKey {
        self.auth.shared_key()
    }

    pub fn session_ttl(&self) -> u32 {
        self.auth.session_ttl()
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
