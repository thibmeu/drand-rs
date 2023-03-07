use anyhow::anyhow;
use std::process;

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
    /// Interact with timelock encryption
    ///
    /// INPUT defaults to standard input (not supported for decryption), and OUTPUT defaults to standard output.
    ///
    /// ROUND can be:
    /// - A specific round. e.g. 123
    /// - A duration. e.g. 30s (not supported)
    /// - An RFC3339 date. e.g. 2023-06-28 21:30:22+00:00 (not supported)
    ///
    /// PATH is a path to a file containing age recipients, one per line
    /// (ignoring "#" prefixed comments and empty lines).
    Crypt {
        /// Encrypt the input (the default)
        #[arg(short, long, default_value_t = true, group = "action")]
        // todo(thibault): add group for armor
        encrypt: bool,
        /// Decrypt the input
        #[arg(short, long, group = "action")]
        decrypt: bool,
        /// Set default upstream. If empty, use the lastest upstream
        #[arg(short = 'u', long, value_hint = ValueHint::Url)]
        set_upstream: Option<String>,
        /// Encrypt to the specified ROUND
        ///
        /// ROUND can be:
        /// a specific round (123),
        /// a duration (30s),
        /// an RFC3339 date (2023-06-28 21:30:22)
        #[arg(short, long)]
        round: Option<String>,
        /// Encrypt to a PEM encoded format
        #[arg(short, long)]
        armor: bool,
        /// Write the result to the file at path OUTPUT
        #[arg(short, long)]
        output: Option<String>,
        #[arg(required = true)]
        /// Path to a file to read from
        input: Option<String>,
    },
    /// Retrieve public randomness
    Rand {
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
    /// Manage set of remote beacon chains
    Remote {
        #[command(subcommand)]
        command: Option<RemoteCommand>,
    },
    /// Prints path to configuration file
    Config {},
}

#[derive(Subcommand)]
enum RemoteCommand {
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
mod time;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut cfg: config::Local = config::Local::load();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let output = match cli.command {
        Commands::Rand {
            set_upstream,
            verify,
            format,
            beacon,
        } => {
            let chain = cfg.set_upstream_and_chain(set_upstream).unwrap();
            cmd::rand(&cfg, format, chain, beacon, verify).await
        }
        Commands::Crypt {
            encrypt: _,
            decrypt,
            set_upstream,
            round,
            armor,
            output,
            input,
        } => {
            let chain = cfg.set_upstream_and_chain(set_upstream).unwrap();
            if decrypt {
                cmd::crypt::decrypt(&cfg, output, input, chain).await
            } else {
                cmd::crypt::encrypt(&cfg, output, input, armor, chain, round).await
            }
        }
        Commands::Remote { command } => match command {
            Some(command) => match command {
                RemoteCommand::Add { name, url } => cmd::remote::add(&mut cfg, name, url).await,
                RemoteCommand::Remove { name } => cmd::remote::remove(&mut cfg, name),
                RemoteCommand::Rename { old, new } => cmd::remote::rename(&mut cfg, old, new),
                RemoteCommand::SetUrl { name, url } => cmd::remote::set_url(&mut cfg, name, url),
                RemoteCommand::Info { format, name } => cmd::remote::info(
                    &cfg,
                    format,
                    name.or(cfg.upstream())
                        .ok_or(anyhow!("No chain or upstream"))
                        .unwrap(),
                ),
            },
            None => cmd::remote::list(&cfg),
        },
        Commands::Config {} => cmd::config(),
    };

    match output {
        Ok(result) => {
            cfg.store().unwrap();
            println!("{result}")
        }
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1)
        }
    }
}
