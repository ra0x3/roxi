#[tokio::main]
async fn main() -> anyhow::Result<()> {
    roxi_cli::run_cli().await
}
