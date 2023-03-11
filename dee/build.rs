#[path = "src/cli.rs"]
mod cli;

use clap_complete::Shell::{Bash, Elvish, Fish, PowerShell, Zsh};
use flate2::{write::GzEncoder, Compression};
use std::io::Write;

const COMPLETIONS_DIR: &str = "../target/completions";
const MANPAGES_DIR: &str = "../target/manpages";

fn create_folder(path: &str) -> std::path::PathBuf {
    let path = std::path::PathBuf::from(path);
    std::fs::create_dir_all(path.clone()).unwrap();
    path
}

fn main() -> std::io::Result<()> {
    let completions_dir = create_folder(COMPLETIONS_DIR);
    let manpages_dir = create_folder(MANPAGES_DIR);

    let cmd = <cli::Cli as clap::CommandFactory>::command();

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    let path = manpages_dir.join("dee.1.gz");
    let file =
        std::fs::File::create(path).expect("Should be able to open file in target directory");
    let mut encoder = GzEncoder::new(file, Compression::best());
    encoder
        .write_all(&buffer)
        .expect("Should be able to write to file in target directory");

    let cmd = &mut <cli::Cli as clap::CommandFactory>::command();
    for shell in [Bash, Elvish, Fish, PowerShell, Zsh] {
        let _path = clap_complete::generate_to(shell, cmd, "dee", completions_dir.clone())?;
    }

    Ok(())
}
