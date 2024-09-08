use crate::{error::ClientError, ClientResult};
use roxi_lib::types::SharedKey;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

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
pub struct Gateway {
    interface: Ipv4Addr,
    port: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoxiServer {
    interface: Ipv4Addr,
    port: u32,
}

impl RoxiServer {
    pub fn hostname(&self) -> String {
        format!("{}:{}", self.interface, self.port)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    roxi_server: RoxiServer,
    auth: Auth,
    gateway: Gateway,
}

impl Config {
    pub fn roxi_server_hostname(&self) -> String {
        self.roxi_server.hostname()
    }

    pub fn shared_key(&self) -> SharedKey {
        self.auth.shared_key()
    }
}

impl TryFrom<&PathBuf> for Config {
    type Error = ClientError;
    fn try_from(p: &PathBuf) -> ClientResult<Self> {
        let file = File::open(p)?;
        let content: serde_yaml::Value = serde_yaml::from_reader(file)?;
        let config: Config = serde_yaml::from_value(content)?;
        Ok(config)
    }
}

impl TryFrom<&Path> for Config {
    type Error = ClientError;
    fn try_from(p: &Path) -> ClientResult<Self> {
        let file = File::open(p)?;
        let content: serde_yaml::Value = serde_yaml::from_reader(file)?;
        let config: Config = serde_yaml::from_value(content)?;
        Ok(config)
    }
}

impl TryFrom<&str> for Config {
    type Error = ClientError;
    fn try_from(s: &str) -> ClientResult<Self> {
        let content: serde_yaml::Value = serde_yaml::from_str(s)?;
        let config: Config = serde_yaml::from_value(content)?;
        Ok(config)
    }
}
