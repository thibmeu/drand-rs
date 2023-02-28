use anyhow::Result;

use drand_client::{
    chain::{self, ChainClient, ChainOptions},
    http_chain_client::HttpChainClient,
};

pub async fn get(url: String, verify: bool, beacon: Option<u64>) -> Result<String> {
    let chain = chain::Chain::new(&url);

    let client = HttpChainClient::new(chain, Some(ChainOptions::new(verify, true, None)));

    let beacon = match beacon {
        Some(round) => client.get(round).await?,
        None => client.latest().await?,
    };

    Ok(serde_json::to_string_pretty(&beacon)?)
}
