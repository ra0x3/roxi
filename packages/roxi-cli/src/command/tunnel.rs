use clap::Parser;
use roxi_client::{Client, Config};
use roxi_lib::util::init_logging;
use std::path::PathBuf;

#[derive(Debug, Parser, Clone)]
#[clap(name = "Roxi client", about = "Roxi client", version)]
pub struct Args {
    /// Config file.
    #[clap(short, long, help = "Config file.")]
    pub config: PathBuf,
}

pub async fn exec(args: Args) -> anyhow::Result<()> {
    let config = Config::try_from(&args.config)?;

    tracing::info!("Configuration: {config:?}");

    init_logging().await?;

    let mut client = Client::new(config).await?;

    if let Err(e) = client.ping().await {
        tracing::error!("Could not ping server: {e}");
    }

    if let Err(e) = client.authenticate().await {
        tracing::error!("Could not authenticate: {e}");
    }

    if let Err(e) = client.request_tunnel_info().await {
        tracing::error!("Could not request tunnel info: {e}");
    }

    if let Err(e) = client.request_stun_info().await {
        tracing::error!("Could not request stun info: {e}");
    }

    if let Err(e) = client.ping().await {
        tracing::error!("Could not ping server: {e}");
    }

    Ok(())
}
