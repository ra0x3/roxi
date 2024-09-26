use crate::{error::ServerError, ServerResult};
use roxi_lib::types::{InterfaceKind, SharedKey, WireGuard};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    net::IpAddr,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Ports {
    tcp: u16,
    udp: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tun {
    address: IpAddr,
    netmask: IpAddr,
    name: String,
    destination: IpAddr,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Auth {
    shared_key: SharedKey,
    session_ttl: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    interface: IpAddr,
    ip: IpAddr,
    ports: Ports,
    max_clients: u16,
}

impl Server {
    pub fn addr(&self, k: InterfaceKind) -> String {
        match k {
            InterfaceKind::Tcp => format!("{}:{}", self.interface, self.ports.tcp),
            InterfaceKind::Udp => format!("{}:{}", self.interface, self.ports.udp),
        }
    }

    pub fn remote_addr(&self, k: InterfaceKind) -> String {
        match k {
            InterfaceKind::Tcp => format!("{}:{}", self.ip, self.ports.tcp),
            InterfaceKind::Udp => format!("{}:{}", self.ip, self.ports.udp),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Network {
    server: Server,
    tun: Tun,
    wireguard: WireGuard,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    network: Network,
    auth: Auth,
    path: PathBuf,
}

impl Config {
    pub fn addr(&self, k: InterfaceKind) -> String {
        self.network.server.addr(k)
    }

    pub fn remote_addr(&self, k: InterfaceKind) -> String {
        self.network.server.remote_addr(k)
    }

    pub fn max_clients(&self) -> u16 {
        self.network.server.max_clients
    }

    pub fn shared_key(&self) -> SharedKey {
        self.auth.shared_key.clone()
    }

    pub fn session_ttl(&self) -> u64 {
        self.auth.session_ttl
    }

    pub fn wireguard(&self) -> WireGuard {
        self.network.wireguard.clone()
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
