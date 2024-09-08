/* TODO: Remove this since we're using P2P for tunneling */
use crate::{config::Config, error::ServerError, ServerResult};
use async_std::sync::Mutex;
use roxi_lib::types::ClientId;
use std::{collections::HashMap, net::Ipv4Addr};

pub struct IpPoolManager {
    available_ips: Mutex<Vec<Ipv4Addr>>,
    assigned_ips: Mutex<HashMap<ClientId, Ipv4Addr>>,
}

impl IpPoolManager {
    pub fn new(_config: Config) -> ServerResult<Self> {
        Ok(IpPoolManager {
            available_ips: Mutex::new(vec![]),
            assigned_ips: Mutex::new(HashMap::new()),
        })
    }

    pub async fn assign_ip(&self, client_id: &ClientId) -> ServerResult<Ipv4Addr> {
        let mut available = self.available_ips.lock().await;
        let mut assigned = self.assigned_ips.lock().await;

        if available.is_empty() {
            tracing::error!("No IP addresses available");
            return Err(ServerError::NoIpAddrAvailable);
        }

        let ip = available.pop().unwrap();
        assigned.insert(client_id.clone(), ip);

        Ok(ip)
    }

    pub async fn release_ip(&self, client_id: &ClientId) -> ServerResult<()> {
        let mut available = self.available_ips.lock().await;
        let mut assigned = self.assigned_ips.lock().await;

        if let Some(ip) = assigned.remove(client_id) {
            available.push(ip);
        }

        Ok(())
    }
}
