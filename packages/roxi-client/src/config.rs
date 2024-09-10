use crate::{error::ClientError, ClientResult};
use roxi_lib::types::SharedKey;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stun {
    ip: Ipv4Addr,
    port: u16,
}

impl From<[u8; 6]> for Stun {
    fn from(d: [u8; 6]) -> Self {
        let ip = Ipv4Addr::new(d[0], d[1], d[2], d[3]);
        let port = u16::from_be_bytes([d[4], d[5]]);
        Self { ip, port }
    }
}

impl Stun {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
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
pub struct Gateway {
    interface: Ipv4Addr,
    port: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoxiServer {
    interface: Ipv4Addr,
    ip: Ipv4Addr,
    port: u32,
}

impl RoxiServer {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn interface(&self) -> String {
        self.interface.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    roxi_server: RoxiServer,
    auth: Auth,
    gateway: Gateway,
    stun: Stun,
}

impl Config {
    pub fn set_stun(&mut self, stun: Stun) {
        self.stun = stun;
    }
    pub fn remote_addr(&self) -> String {
        self.roxi_server.addr()
    }

    pub fn shared_key(&self) -> SharedKey {
        self.auth.shared_key()
    }

    pub fn stun_addr(&self) -> String {
        self.stun.addr()
    }

    pub fn udp_bind(&self) -> String {
        format!("{}:{}", self.roxi_server.interface(), 59600)
    }

    pub fn save(&self, p: PathBuf) -> ClientResult<()> {
        let content = serde_yaml::to_string(&self)?;
        let mut f = File::create(p)?;
        f.write_all(content.as_bytes())?;
        Ok(())
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
