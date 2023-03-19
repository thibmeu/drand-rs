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
pub struct Cli {
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interact with timelock encryption
    ///
    /// INPUT defaults to standard input, and OUTPUT defaults to standard output.
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
    /// BEACON defaults to the latest beacon, and FORMAT to long.
    ///
    /// UPSTREAM is an existing remote, and defaults to the lastest used.
    ///
    /// Example:
    ///     $ dee rand -u myremote 1000
    ///     $ dee rand -f long
    #[command(verbatim_doc_comment)]
    Rand {
        /// Set default upstream. If empty, use the lastest upstream.
        #[arg(short = 'u', long, value_hint = ValueHint::Url)]
        set_upstream: Option<String>,
        /// Enable beacon response validation.
        #[arg(long, default_value_t = true)]
        verify: bool,
        /// Enable detailed output
        #[arg(short, long, default_value_t = false, group = "format")]
        long: bool,
        /// Enable json output, as defined per drand API
        #[arg(long, default_value_t = false, group = "format")]
        json: bool,
        /// Round number to retrieve. Leave empty to retrieve the latest round.
        beacon: Option<u64>,
    },
    /// Manage set of tracked chains.
    ///
    /// With no arguments, shows a list of existing remotes. Several subcommands are available to perform operations on the remotes.
    ///
    /// With the -v option, remote URLs are shown as well.
    Remote {
        #[command(subcommand)]
        command: Option<RemoteCommand>,
    },
}

#[derive(Subcommand)]
pub enum RemoteCommand {
    /// Add a remote named <name> for the chain at <URL>. The command dee rand -u <name> can then be used to create and update remote-tracking chain <name>.
    ///
    /// By default, only information on managed chains are imported.
    Add { name: String, url: String },
    /// Rename the remote named <old> to <new>. The remote-tracking chain and configuration settings for the remote are updated.
    Rename { old: String, new: String },
    /// Remove the remote named <name>. The remote-tracking chain and configuration settings for the remote are removed.
    Remove { name: String },
    /// Change URLs for the remote.
    SetUrl { name: String, url: String },
    /// Give some information about the remote <name>.
    Show {
        /// Enable detailed output
        #[arg(short, long, default_value_t = false, group = "format")]
        long: bool,
        /// Enable json output, as defined per drand API
        #[arg(long, default_value_t = false, group = "format")]
        json: bool,
        name: Option<String>,
    },
}

#[allow(dead_code)]
pub fn build() -> Cli {
    Cli::parse()
}
