use clap::Parser;
use roxi_lib::util::{init_logging, shutdown_signal_handler};
use roxi_server::{Config, Server};
use std::{path::PathBuf, sync::Arc};
use tokio::{sync::broadcast, task::JoinSet};

#[derive(Debug, Parser, Clone)]
#[clap(name = "Roxi server", about = "Roxi server", version)]
pub struct Args {
    /// Config file.
    #[clap(short, long, help = "Config file.")]
    pub config: PathBuf,
}

pub async fn exec(args: Args) -> anyhow::Result<()> {
    let (tx, _rx) = broadcast::channel::<()>(1);

    let mut subsystems: JoinSet<()> = JoinSet::new();
    subsystems.spawn(shutdown_signal_handler()?);

    let config = Config::try_from(&args.config)?;

    tracing::info!("Configuration: {config:?}");
    let server = Arc::new(Server::new(config).await?);

    init_logging().await?;

    subsystems.spawn(async move {
        let tcp = server.clone().run();
        let udp = server.clone().run_stun();

        let _ = tokio::join!(tcp, udp);

        if let Err(e) = server.run_stun().await {
            tracing::error!("Stun server failed: {e}");
        }

        if let Err(e) = tx.send(()) {
            tracing::error!("Failed to send shutdown signal: {e}");
        }
    });

    if subsystems.join_next().await.is_some() {
        subsystems.shutdown().await;
    }

    Ok(())
}
