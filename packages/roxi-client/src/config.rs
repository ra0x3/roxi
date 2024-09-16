use crate::{error::ClientError, ClientResult};
use roxi_lib::types::{InterfaceKind, SharedKey};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Ports {
    tcp: u16,
    udp: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Stun {
    ip: Option<Ipv4Addr>,
    port: Option<u16>,
}

impl From<[u8; 6]> for Stun {
    fn from(d: [u8; 6]) -> Self {
        let ip = Ipv4Addr::new(d[0], d[1], d[2], d[3]);
        let port = u16::from_be_bytes([d[4], d[5]]);
        Self {
            ip: Some(ip),
            port: Some(port),
        }
    }
}

impl Stun {
    pub fn addr(&self) -> Option<String> {
        if self.ip.is_some() && self.port.is_some() {
            return Some(format!("{}:{}", self.ip.unwrap(), self.port.unwrap()));
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Auth {
    shared_key: SharedKey,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Nat {
    attempts: u8,
    delay: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Gateway {
    interface: Ipv4Addr,
    ip: Ipv4Addr,
    ports: Ports,
    max_clients: u16,
}

// FIXME: Maybe bind these common methods with a tait?
impl Gateway {
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

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Server {
    interface: Ipv4Addr,
    ip: Ipv4Addr,
    ports: Ports,
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

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Network {
    server: Server,
    gateway: Gateway,
    stun: Stun,
    wireguard_config: PathBuf,
    nat: Nat,
}

impl Network {
    pub fn set_stun(&mut self, stun: Stun) {
        self.stun = stun;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Config {
    auth: Auth,
    path: PathBuf,
    network: Network,
}

impl Config {
    pub fn set_stun(&mut self, stun: Stun) {
        self.network.set_stun(stun);
    }

    pub fn addr(&self, k: InterfaceKind) -> String {
        self.network.server.addr(k)
    }

    pub fn remote_addr(&self, k: InterfaceKind) -> String {
        self.network.server.remote_addr(k)
    }

    pub fn stun_addr(&self) -> anyhow::Result<String> {
        match self.network.stun.addr() {
            Some(addr) => Ok(addr),
            None => Err(anyhow::anyhow!("STUN address expected")),
        }
    }

    pub fn shared_key(&self) -> SharedKey {
        self.auth.shared_key.clone()
    }

    pub fn gateway_addr(&self, k: InterfaceKind) -> String {
        self.network.gateway.addr(k)
    }

    pub fn gateway_remote_addr(&self, k: InterfaceKind) -> String {
        self.network.gateway.remote_addr(k)
    }

    pub fn max_gateway_clients(&self) -> u16 {
        self.network.gateway.max_clients
    }

    pub fn wireguard_config(&self) -> &PathBuf {
        &self.network.wireguard_config
    }

    pub fn nat_punch_delay(&self) -> u8 {
        self.network.nat.delay
    }

    pub fn nat_punch_attempts(&self) -> u8 {
        self.network.nat.attempts
    }

    pub fn save(&self) -> ClientResult<()> {
        let content = serde_yaml::to_string(&self)?;
        let mut f = File::create(&self.path)?;
        f.write_all(content.as_bytes())?;
        Ok(())
    }
}

impl TryFrom<Vec<u8>> for Config {
    type Error = serde_yaml::Error;
    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        let c: Config = serde_yaml::from_slice(&v)?;
        Ok(c)
    }
}

impl TryFrom<Config> for Vec<u8> {
    type Error = serde_yaml::Error;
    fn try_from(c: Config) -> Result<Self, Self::Error> {
        let v = serde_yaml::to_string(&c)?;
        Ok(v.into_bytes())
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
