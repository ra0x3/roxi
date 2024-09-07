use serde::{Serialize, Deserialize};
use std::net::Ipv4Addr;

#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
    limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IP {
    pool: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tun {
   address: Ipv4Addr,
   netmask: Ipv4Addr,
   name: String,
   destination: Ipv4Addr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    expiry: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    client: Client,
    ip: IP,
    tun: Tun,
}
