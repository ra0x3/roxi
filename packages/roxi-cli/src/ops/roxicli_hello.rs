use crate::commands::hello::Command;

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

#[allow(unused)]
pub fn init(command: Command) -> anyhow::Result<()> {
    print_welcome_message();
    Ok(())
}
