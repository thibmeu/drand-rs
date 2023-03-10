#[path = "src/cli.rs"]
mod cli;

use clap_complete::Shell::{Bash, Elvish, Fish, PowerShell, Zsh};

fn main() -> std::io::Result<()> {
    let out_dir =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?);

    let cmd = <cli::Cli as clap::CommandFactory>::command();

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    let path = out_dir.join("dee.1");
    std::fs::write(path, buffer)?;

    let cmd = &mut <cli::Cli as clap::CommandFactory>::command();
    for shell in [Bash, Elvish, Fish, PowerShell, Zsh] {
        let _path = clap_complete::generate_to(
            shell,
            cmd,        // We need to specify what generator to use
            "dee",           // We need to specify the bin name manually
            out_dir.clone(), // We need to specify where to write to
        )?;
    }

    Ok(())
}
