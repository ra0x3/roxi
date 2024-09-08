use sha2::{Digest, Sha256};
use std::{env, str::FromStr};
use tokio::{
    signal::unix::{signal, Signal, SignalKind},
    sync::broadcast,
};
use tracing_subscriber::filter::EnvFilter;

const RUST_LOG: &str = "RUST_LOG";
const HUMAN_LOGGING: &str = "HUMAN_LOGGING";

pub async fn init_logging() -> anyhow::Result<()> {
    let level = env::var_os(RUST_LOG)
        .map(|x| x.into_string().unwrap())
        .unwrap_or("info".to_string());

    let filter = match env::var_os(RUST_LOG) {
        Some(_) => {
            EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
        }
        None => EnvFilter::new("info"),
    };

    let human_logging = env::var_os(HUMAN_LOGGING)
        .map(|s| {
            bool::from_str(s.to_str().unwrap())
                .expect("Expected `true` or `false` to be provided for `HUMAN_LOGGING`")
        })
        .unwrap_or(true);

    let sub = tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter);

    if human_logging {
        sub.with_ansi(true)
            .with_level(true)
            .with_line_number(true)
            .init();
    } else {
        sub.with_ansi(false)
            .with_level(true)
            .with_line_number(true)
            .json()
            .init();
    }
    Ok(())
}

pub fn shutdown_signal_handler() -> std::io::Result<impl futures::Future<Output = ()>> {
    let mut sighup: Signal = signal(SignalKind::hangup())?;
    let mut sigterm: Signal = signal(SignalKind::terminate())?;
    let mut sigint: Signal = signal(SignalKind::interrupt())?;

    let future = async move {
        #[cfg(unix)]
        {
            tokio::select! {
                _ = sighup.recv() => {
                    tracing::info!("Received SIGHUP. Stopping services.");
                }
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM. Stopping services.");
                }
                _ = sigint.recv() => {
                    tracing::info!("Received SIGINT. Stopping services.");
                }
            }
        }

        #[cfg(not(unix))]
        {
            signal::ctrl_c().await?;
            tracing::info!("Received CTRL+C. Stopping services.");
        }
    };

    Ok(future)
}

pub fn sha256(input: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    hex::encode(result)
}
