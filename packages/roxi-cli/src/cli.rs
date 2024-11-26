pub(crate) use crate::command::{
    auth, gateway, ping, quick, request_gateway, seed, serve, stun, tunnel,
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
    #[clap(name = "serve", about = "Start a ROXI server.")]
    Serve(serve::Args),
    #[clap(name = "ping", about = "Ping a ROXI server.")]
    Ping(ping::Args),
    #[clap(name = "auth", about = "Authenticate against ROXI server.")]
    Auth(auth::Args),
    #[clap(name = "stun", about = "Send public IP to STUN server.")]
    Stun(stun::Args),
    #[clap(name = "gateway", about = "Start ROXI gateway server.")]
    Gateway(gateway::Args),
    #[clap(name = "quick", about = "Run wg-quick.")]
    Quick(quick::Args),
    #[clap(name = "seed", about = "Seed a client against the server.")]
    Seed(seed::Args),
    #[clap(name = "request_gateway", about = "Request a gateway from the server.")]
    RequestGateway(request_gateway::Args),
    #[clap(name = "tunnel", about = "Create tunnel a tunnel between two peers")]
    Tunnel(tunnel::Args),
}

pub async fn run_cli() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    match opt.command {
        RoxiCli::Serve(command) => serve::exec(command).await,
        RoxiCli::Ping(command) => ping::exec(command).await,
        RoxiCli::Auth(command) => auth::exec(command).await,
        RoxiCli::Stun(command) => stun::exec(command).await,
        RoxiCli::Gateway(command) => gateway::exec(command).await,
        RoxiCli::Quick(command) => quick::exec(command).await,
        RoxiCli::Seed(command) => seed::exec(command).await,
        RoxiCli::RequestGateway(command) => request_gateway::exec(command).await,
        RoxiCli::Tunnel(command) => tunnel::exec(command).await,
    }
}
