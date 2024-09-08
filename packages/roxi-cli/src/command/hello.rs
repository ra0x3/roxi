use anyhow::Result;
use clap::Parser;

/// Say hello from the Roxi project
#[derive(Debug, Parser)]
pub struct Args {
    /// Enable verbose output.
    #[clap(short, long, help = "Enable verbose output.")]
    pub verbose: bool,
}

#[allow(unused)]
pub fn exec(args: Args) -> Result<()> {
    fn print_welcome_message() {
        let msg = r#"
.______        ______   ___   ___  __
|   _  \      /  __  \  \  \ /  / |  |
|  |_)  |    |  |  |  |  \  V  /  |  |
|      /     |  |  |  |   >   <   |  |
|  |\  \----.|  `--'  |  /  .  \  |  |
| _| `._____| \______/  /__/ \__\ |__|
"#;

        println!("{msg}");
    }
    print_welcome_message();
    Ok(())
}
