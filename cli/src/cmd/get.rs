use anyhow::Result;

use colored::Colorize;
use drand_client::{
    beacon::RandomnessBeacon,
    chain::{self, ChainClient, ChainOptions},
    http_chain_client::HttpChainClient,
};

use crate::print::{print_with_format, Format, Print};

impl Print for RandomnessBeacon {
    fn pretty(&self) -> Result<String> {
        Ok(format!(
            r#"{}: {}
{}: {}
{}: {}"#,
            "Round".bold(),
            self.round(),
            "Randomness".bold(),
            self.randomness(),
            "Signature".bold(),
            self.signature()
        ))
    }

    fn json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

pub async fn get(url: String, verify: bool, format: Format, beacon: Option<u64>) -> Result<String> {
    let chain = chain::Chain::new(&url);

    let client = HttpChainClient::new(chain, Some(ChainOptions::new(verify, true, None)));

    let beacon = match beacon {
        Some(round) => client.get(round).await?,
        None => client.latest().await?,
    };

    Ok(print_with_format(beacon, format)?)
}
