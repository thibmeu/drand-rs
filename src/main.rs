use anyhow::Result;

use drand_client::{chain::{self, ChainClient}, http_chain_client};

#[tokio::main]
async fn main() -> Result<()> {
    let chain = chain::Chain::new("https://drand.cloudflare.com");

    let mut client = http_chain_client::HttpChainClient::new(chain, None);

    let latest = client.latest().await?;

    println!("{:#?}", latest);
    Ok(())
}
