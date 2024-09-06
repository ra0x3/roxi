use crate::ops::roxicli_hello;
use anyhow::Result;
use clap::Parser;

/// Say hello from the Roxi project
#[derive(Debug, Parser)]
pub struct Command {
    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.")]
    pub verbose: bool,
}

pub fn exec(command: Command) -> Result<()> {
    roxicli_hello::init(command)?;
    Ok(())
}
