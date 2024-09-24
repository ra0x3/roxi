pub(crate) use crate::command::{
    auth, connect, gateway, hello, ping, quick, serve, stun,
};
use clap::{Parser, Subcommand};

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
    #[clap(name = "ping", about = "Ping Roxi server")]
    Ping(ping::Args),
    #[clap(name = "auth", about = "Authenticate against Roxi server")]
    Auth(auth::Args),
    #[clap(name = "stun", about = "Send public IP to STUN server")]
    Stun(stun::Args),
    #[clap(name = "gateway", about = "Start Roxi gateway server")]
    Gateway(gateway::Args),
    #[clap(name = "quick", about = "Run wg-quick")]
    Quick(quick::Args),
}

pub async fn run_cli() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    match opt.command {
        RoxiCli::Hello(command) => hello::exec(command),
        RoxiCli::Serve(command) => serve::exec(command).await,
        RoxiCli::Connect(command) => connect::exec(command).await,
        RoxiCli::Ping(command) => ping::exec(command).await,
        RoxiCli::Auth(command) => auth::exec(command).await,
        RoxiCli::Stun(command) => stun::exec(command).await,
        RoxiCli::Gateway(command) => gateway::exec(command).await,
        RoxiCli::Quick(command) => quick::exec(command).await,
    }
}
