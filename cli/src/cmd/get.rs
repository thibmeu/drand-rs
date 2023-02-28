use anyhow::Result;

use drand_client::{chain, http_chain_client};

pub async fn get(url: String, beacon: Option<u64>) -> Result<String> {
    let chain = chain::Chain::new(&url);

    use chain::ChainClient;
    let client = http_chain_client::HttpChainClient::new(chain, None);

    let beacon = match beacon {
        Some(round) => client.get(round).await?,
        None => client.latest().await?,
    };

    Ok(serde_json::to_string_pretty(&beacon)?)
}
