use anyhow::Result;

use colored::Colorize;
use drand_client::{
    beacon::RandomnessBeacon,
    chain::{self, ChainClient, ChainOptions, ChainVerification},
    http_chain_client::HttpChainClient,
};

use crate::{
    config::ConfigChain,
    print::{print_with_format, Format, Print},
};

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

pub async fn get(
    chain: ConfigChain,
    verify: bool,
    format: Format,
    beacon: Option<u64>,
) -> Result<String> {
    let chain = chain::Chain::new(&chain.url());
    let info = chain.info().await?;

    let client = HttpChainClient::new(
        chain,
        Some(ChainOptions::new(
            verify,
            true,
            Some(ChainVerification::new(
                Some(info.hash()),
                Some(info.public_key()),
            )),
        )),
    );

    let beacon = match beacon {
        Some(round) => client.get(round).await?,
        None => client.latest().await?,
    };

    print_with_format(beacon, format)
}
