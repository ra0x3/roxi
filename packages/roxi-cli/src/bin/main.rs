use tracing::error;

#[tokio::main]
async fn main() {
    if let Err(err) = roxi_cli::cli::run_cli().await {
        error!("Error: {:?}", err);
        std::process::exit(1);
    }
}
