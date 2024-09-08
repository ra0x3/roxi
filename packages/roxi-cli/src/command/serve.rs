use clap::Parser;
use roxi_lib::util::{init_logging, shutdown_signal_handler};
use roxi_server::{Config, Server};
use std::path::PathBuf;
use tokio::{
    sync::{broadcast, mpsc::channel},
    task::JoinSet,
};

#[derive(Debug, Parser, Clone)]
#[clap(name = "Roxi server", about = "Roxi server", version)]
pub struct Args {
    /// Service config file.
    #[clap(short, long, help = "Service config file.")]
    pub config: PathBuf,
}

pub async fn exec(args: Args) -> anyhow::Result<()> {
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

    let mut subsystems: JoinSet<()> = JoinSet::new();
    subsystems.spawn(shutdown_signal_handler()?);

    let config = Config::try_from(&args.config)?;

    tracing::info!("Configuration: {:?}", config);

    init_logging().await?;

    subsystems.spawn(async move {
        if let Err(e) = Server::new(config).await {
            tracing::error!("Server failed: {e}");
        }

        if let Err(e) = shutdown_tx.send(()) {
            tracing::error!("Failed to send shutdown signal: {e}");
        }
    });

    if subsystems.join_next().await.is_some() {
        subsystems.shutdown().await;
    }

    Ok(())
}
