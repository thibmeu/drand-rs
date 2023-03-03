use std::process;
use anyhow::anyhow;

use clap::{Parser, Subcommand, ValueHint};

/// 1. First interaction
/// drand get --url https://drand.cloudflare.com # latest beacon
/// drand get --url https://drand.cloudflare.com 100 # round 100
/// drand get --url https://drand.cloudflare.com 2022...Z # round time from UTC
/// drand
/// 2. Second allow disabling verification
/// drand get --verify=false --chain-url https://drand.cloudflare.com # disable beacon verification
/// 3. Chain management
/// drand chain add cloudflare https://drand.cloudflare.com # add chain to local configuration
/// drand chain set-url cloudflare https://drand.cloudflare.com
/// drand chain # list all chains
/// drand chain info cloudflare
/// drand chain info --cache=false cloudflare # chain is cached locally for validation
/// 4. Active drand node
/// drand watch cloudflare

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Retrieve public randomness
    Get {
        /// Set default upstream. If empty, use the lastest upstream
        #[arg(short = 'u', long, value_hint = ValueHint::Url)]
        set_upstream: Option<String>,
        /// Enable beacon response validation
        #[arg(long, default_value_t = true)]
        verify: bool,
        /// Output format
        #[arg(long, value_enum, default_value_t = print::Format::Pretty)]
        format: print::Format,
        /// Round number to retrieve. Leave empty to retrieve the latest round
        beacon: Option<u64>,
    },
    /// Manage set of beacon chains
    Chain {
        #[command(subcommand)]
        command: Option<ChainCommand>,
    },
    /// Prints path to configuration file
    Config {},
}

#[derive(Subcommand)]
enum ChainCommand {
    /// Add remote chain
    Add { name: String, url: String },
    /// Remove remote chain
    Remove { name: String },
    /// Rename remote chain
    Rename { old: String, new: String },
    /// Set URL for remote chain
    SetUrl { name: String, url: String },
    /// Retrieve and store info about remote chain
    Info {
        /// Output format
        #[arg(long, value_enum, default_value_t = print::Format::Pretty)]
        format: print::Format,
        name: Option<String>,
    },
}

mod cmd;
mod config;
mod print;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut cfg: config::Local = config::Local::load();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let output = match cli.command {
        Commands::Get {
            set_upstream,
            verify,
            format,
            beacon,
        } => {
            let chain = cfg.set_upstream_and_chain(set_upstream).unwrap();
            cmd::get(&cfg, format, chain, beacon, verify).await
        }
        Commands::Chain { command } => match command {
            Some(command) => match command {
                ChainCommand::Add { name, url } => cmd::chain::add(&mut cfg, name, url).await,
                ChainCommand::Remove { name } => cmd::chain::remove(&mut cfg, name),
                ChainCommand::Rename { old, new } => cmd::chain::rename(&mut cfg, old, new),
                ChainCommand::SetUrl { name, url } => cmd::chain::set_url(&mut cfg, name, url),
                ChainCommand::Info { format, name } => cmd::chain::info(&cfg, format, name.or(cfg.upstream()).ok_or(anyhow!("No chain or upstream")).unwrap()),
            },
            None => cmd::chain::list(&cfg),
        },
        Commands::Config {} => cmd::config(),
    };

    match output {
        Ok(result) => {
            cfg.store().unwrap();
            println!("{result}")
        }
        Err(err) => {
            eprintln!("{err}");
            process::exit(1)
        }
    }
}
