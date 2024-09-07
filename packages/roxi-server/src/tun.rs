use crate::ServerResult;
use std::{
    io::{Read, Write},
    net::Ipv4Addr,
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
    task,
};
use tun::{platform::Device as TunDevice, Configuration, Device};

struct TunInterface;

impl TunInterface {
    pub async fn new() -> ServerResult<()> {
        let mut config = Configuration::default();
        config
            .address(Ipv4Addr::new(10, 0, 0, 1))
            .destination(Ipv4Addr::new(10, 0, 0, 2))
            .netmask(Ipv4Addr::new(255, 255, 255, 0))
            .name("utun0");

        let mut dev = Arc::new(Mutex::new(TunDevice::new(&config)?));
        tracing::info!("TUN interface created: {:?}", dev.blocking_lock().name());

        loop {
            let mut buff = vec![0u8; 1500];
            let dev = Arc::clone(&dev);

            let n = task::spawn_blocking(move || {
                let mut dev = dev.blocking_lock();
                dev.read(&mut buff)
            })
            .await??;

            tracing::info!("Read {n} bytes from TUN interface");

            //task::spawn_blocking(move || dev.write_all(&buff[..n])).await??;
        }
    }
}
