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
    /// * a specific round (123),
    /// * a duration (30s),
    /// * an RFC3339 date (2023-06-28 21:30:22)
    ///
    /// UPSTREAM is an existing remote, and defaults to the lastest used.
    /// 
    /// Example:
    ///     $ tar cvz ~/data | dee crypt -u myremote -r 30s > data.tar.gz.age
    ///     $ dee crypt --decrypt -o data.tar.gz data.tar.gz.age
    #[command(verbatim_doc_comment)]
    Crypt {
        /// Encrypt the input (the default).
        #[arg(short, long, default_value_t = true, group = "action")]
        // todo(thibault): add group for armor
        encrypt: bool,
        /// Decrypt the input.
        #[arg(short, long, group = "action")]
        decrypt: bool,
        /// Set default upstream. If empty, use the latest upstream.
        #[arg(short = 'u', long, value_hint = ValueHint::Url)]
        set_upstream: Option<String>,
        /// Encrypt to the specified ROUND.
        /// ROUND can be:
        /// * a specific round. e.g. 123,
        /// * a duration. e.g. 30s,
        /// * an RFC3339 date. e.g. 2023-06-28 21:30:22
        #[arg(short, long, verbatim_doc_comment)]
        round: Option<String>,
        /// Encrypt to a PEM encoded format.
        #[arg(short, long)]
        armor: bool,
        /// Write the result to the file at path OUTPUT.
        #[arg(short, long)]
        output: Option<String>,
        /// Path to a file to read from.
        input: Option<String>,
    },
    /// Retrieve public randomness.
    /// 
    /// BEACON defaults to the latest beacon, and FORMAT to pretty.
    /// 
    /// UPSTREAM is an existing remote, and defaults to the lastest used.
    /// 
    /// Example:
    ///     $ dee rand -u myremote 1000
    ///     $ dee rand
    #[command(verbatim_doc_comment)]
    Rand {
        /// Set default upstream. If empty, use the lastest upstream.
        #[arg(short = 'u', long, value_hint = ValueHint::Url)]
        set_upstream: Option<String>,
        /// Enable beacon response validation.
        #[arg(long, default_value_t = true)]
        verify: bool,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = print::Format::Pretty)]
        format: print::Format,
        /// Round number to retrieve. Leave empty to retrieve the latest round.
        beacon: Option<u64>,
    },
    /// Manage set of chains ("remote") whose beacons you track.
    /// 
    ///  With no arguments, shows a list of existing remotes. Several subcommands are available to perform operations on the remotes.
    /// 
    /// With the -v option, remote URLs are shown as well.
    Remote {
        #[command(subcommand)]
        command: Option<RemoteCommand>,
    },
    /// Print path to configuration file.
    Config {},
}

#[derive(Subcommand)]
enum RemoteCommand {
    /// Add a remote named <name> for the chain at <URL>. The command dee rand -u <name> can then be used to create and update remote-tracking chain <name>.
    ///
    /// By default, only information on managed chains are imported.
    Add { name: String, url: String },
    /// Rename the remote named <old> to <new>. The remote-tracking chain and configuration settings for the remote are updated.
    Rename { old: String, new: String },
    /// Remove the remote named <name>. The remote-tracking chain and configuration settings for the remote are removed.
    Remove { name: String },
    /// Changes URLs for the remote.
    SetUrl { name: String, url: String },
    /// Gives some information about the remote <name>.
    Show {
        /// Output format
        #[arg(short, long, value_enum, default_value_t = print::Format::Pretty)]
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
                RemoteCommand::Show { format, name } => cmd::remote::show(
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
