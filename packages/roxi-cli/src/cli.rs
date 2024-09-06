#[allow(unused)]
pub(crate) use crate::commands::{hello::Command as HelloCommand};
use clap::{Parser, Subcommand};
use forc_tracing::{init_tracing_subscriber, TracingSubscriberOptions};

#[derive(Debug, Parser)]
#[clap(name = "Roxi", about = "Roxi Orchestrator", version)]
pub struct Opt {
    /// The command to run
    #[clap(subcommand)]
    pub command: RoxiCLI,
}

#[derive(Subcommand, Debug)]
pub enum RoxiCLI {
    #[clap(name = "hello", about = "Test hello command")]
    Hello(HelloCommand),
}

pub async fn run_cli() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    let tracing_options = TracingSubscriberOptions {
        ..Default::default()
    };
    init_tracing_subscriber(tracing_options);

    match opt.command {
        RoxiCLI::Hello(command) => crate::commands::hello::exec(command),
    }
}
