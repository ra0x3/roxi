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
use tun::{platform::Device as PlatformDevice, Configuration, Device as TunDevice};

pub trait RoxiDevice {
    fn read(&mut self, buff: &mut [u8]) -> ServerResult<usize>;
    fn write(&mut self, buff: &[u8]) -> ServerResult<()>;
    fn name(&self) -> String;
}

pub struct TunInterface {
    device: Arc<Mutex<PlatformDevice>>,
}

impl TunInterface {
    pub fn new() -> ServerResult<Self> {
        let mut config = Configuration::default();
        config
            .address(Ipv4Addr::new(10, 0, 0, 1))
            .destination(Ipv4Addr::new(10, 0, 0, 2))
            .netmask(Ipv4Addr::new(255, 255, 255, 0))
            .name("utun6");

        let device = PlatformDevice::new(&config)?;
        tracing::info!("TUN interface created: {:?}", device.name());
        Ok(Self {
            device: Arc::new(Mutex::new(device)),
        })
    }

    pub async fn run(&self) -> ServerResult<()> {
        loop {
            let buff = Arc::new(Mutex::new(vec![0u8; 1500]));
            let buf = Arc::clone(&buff);
            let dev = Arc::clone(&self.device);

            let n = task::spawn_blocking(move || {
                let mut dev = dev.blocking_lock();
                let mut buff = buf.blocking_lock();
                dev.read(&mut buff)
            })
            .await??;

            tracing::info!("Read {n} bytes from TUN interface");

            let dev = Arc::clone(&self.device);
            let buf = Arc::clone(&buff);
            task::spawn_blocking(move || {
                let mut dev = dev.blocking_lock();
                let buff = buf.blocking_lock();
                dev.write(&buff[..n])
            })
            .await??;
        }
    }
}
