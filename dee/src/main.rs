use anyhow::anyhow;
use std::process;

mod cli;
mod cmd;
mod config;
mod print;
mod time;

#[tokio::main]
async fn main() {
    let cli = cli::build();
    let mut cfg: config::Local = config::Local::load();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let output = match cli.command {
        cli::Commands::Rand {
            set_upstream,
            verify,
            long,
            json,
            beacon,
        } => match cfg.set_upstream_and_chain(set_upstream) {
            Ok(chain) => {
                cmd::rand(&cfg, print::Format::new(long, json), chain, beacon, verify).await
            }
            Err(err) => Err(err),
        },
        cli::Commands::Crypt {
            encrypt: _,
            decrypt,
            set_upstream,
            round,
            armor,
            output,
            input,
        } => match cfg.set_upstream_and_chain(set_upstream) {
            Ok(chain) => {
                if decrypt {
                    cmd::crypt::decrypt(&cfg, output, input, chain).await
                } else {
                    cmd::crypt::encrypt(&cfg, output, input, armor, chain, round).await
                }
            }
            Err(err) => Err(err),
        },
        cli::Commands::Remote { command } => match command {
            Some(command) => match command {
                cli::RemoteCommand::Add { name, url } => {
                    cmd::remote::add(&mut cfg, name, url).await
                }
                cli::RemoteCommand::Remove { name } => cmd::remote::remove(&mut cfg, name),
                cli::RemoteCommand::Rename { old, new } => cmd::remote::rename(&mut cfg, old, new),
                cli::RemoteCommand::SetUrl { name, url } => {
                    cmd::remote::set_url(&mut cfg, name, url)
                }
                cli::RemoteCommand::Show { long, json, name } => cmd::remote::show(
                    &cfg,
                    print::Format::new(long, json),
                    name.or(cfg.upstream())
                        .ok_or(anyhow!("No chain or upstream"))
                        .unwrap(),
                ),
            },
            None => cmd::remote::list(&cfg),
        },
    };

    match output {
        Ok(result) => {
            cfg.store().unwrap();
            if !result.is_empty() {
                println!("{result}")
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1)
        }
    }
}
