use clap::Parser;
use std::path::PathBuf;
use std::process::Command;
use strum_macros::{EnumString, Display};

#[derive(Debug, EnumString, Display, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Action {
    Up,
    Down,
}

/// Control WireGuard using wg-quick
#[derive(Debug, Parser)]
pub struct Args {
    /// Action to perform: up or down
    #[clap(help = "Action to perform (up or down)")]
    pub action: Action,

    /// Path to the wg0.conf file
    #[clap(long, help = "Path to the WireGuard config file.")]
    pub config: PathBuf,

    /// Path to bash (default is /bin/bash)
    #[clap(long, help = "Path to bash executable.", default_value = "/bin/bash")]
    pub bash: PathBuf,
}

pub fn exec(args: Args) -> anyhow::Result<()> {
    let action = args.action.to_string();
    let output = Command::new(args.bash)
        .arg("-c")
        .arg(format!("wg-quick {} {}", action, args.config.display()))
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to run wg-quick: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);

    Ok(())
}
