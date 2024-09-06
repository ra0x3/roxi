use crate::{error::ServerError, ServerResult};
use async_std::sync::Mutex;
use std::{collections::HashMap, net::IpAddr};

pub struct IpPoolManager {
    available_ips: Mutex<Vec<IpAddr>>,
    assigned_ips: Mutex<HashMap<String, IpAddr>>,
}

impl IpPoolManager {
    pub fn new() -> ServerResult<Self> {
        // TODO: Available IPs should come from a global server config
        let available_ips = vec![
            "192.168.1.2".parse()?,
            "192.168.1.3".parse()?,
            "192.168.1.4".parse()?,
        ];

        Ok(IpPoolManager {
            available_ips: Mutex::new(available_ips),
            assigned_ips: Mutex::new(HashMap::new()),
        })
    }

    pub async fn assign_ip(&self, client_id: &str) -> ServerResult<IpAddr> {
        let mut available = self.available_ips.lock().await;
        let mut assigned = self.assigned_ips.lock().await;

        if available.is_empty() {
            tracing::error!("No IP addresses available");
            return Err(ServerError::NoIpAddrAvailable);
        }

        let ip = available.pop().unwrap();
        assigned.insert(client_id.to_string(), ip);

        Ok(ip)
    }

    pub async fn release_ip(&self, client_id: &str) -> ServerResult<()> {
        let mut available = self.available_ips.lock().await;
        let mut assigned = self.assigned_ips.lock().await;

        if let Some(ip) = assigned.remove(client_id) {
            available.push(ip);
        }

        Ok(())
    }
}
