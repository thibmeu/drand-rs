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
/// 4. Active drand node
/// drand watch cloudflare

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Retrieve public randomness
    Get {
        /// Address of the beacon
        #[arg(long, value_hint = ValueHint::Url)]
        url: String,
        /// Address of the beacon
        #[arg(long)]
        verify: bool,
        /// Round number to retrieve. Leave empty to retrieve the latest round
        beacon: Option<u64>,
    },
}

mod cmd;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let output = match cli.command {
        Commands::Get {
            url,
            verify,
            beacon,
        } => cmd::get(url, verify, beacon).await,
    };

    match output {
        Ok(result) => println!("{result}"),
        Err(err) => {
            eprintln!("{err}");
            process::exit(1)
        }
    }
}
