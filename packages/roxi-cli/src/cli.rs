pub(crate) use crate::command::{connect, hello, serve};
use clap::{Parser, Subcommand};
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions};

#[derive(Debug, Parser)]
#[clap(name = "Roxi", about = "Roxi Orchestrator", version)]
pub struct Opt {
    /// The command to run
    #[clap(subcommand)]
    pub command: RoxiCli,
}

#[derive(Subcommand, Debug)]
pub enum RoxiCli {
    #[clap(name = "hello", about = "Test hello command")]
    Hello(hello::Args),
    #[clap(name = "serve", about = "Start Roxi server")]
    Serve(serve::Args),
    #[clap(name = "connect", about = "Connect to Roxi server")]
    Connect(connect::Args),
}

pub async fn run_cli() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    let tracing_options = TracingSubscriberOptions {
        ..Default::default()
    };
    init_tracing_subscriber(tracing_options);

    match opt.command {
        RoxiCli::Hello(command) => hello::exec(command),
        RoxiCli::Serve(command) => serve::exec(command).await,
        RoxiCli::Connect(command) => connect::exec(command).await,
    }
}
