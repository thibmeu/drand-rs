use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tokio::time;

use crate::{chain::{Chain, ChainOptions, ChainClient, ChainInfo}, beacon::RandomnessBeacon};

pub struct HttpChainClient {
  chain: Chain,
  options: ChainOptions,
  cached_chain_info: Option<ChainInfo>,
}

impl HttpChainClient {
  pub fn new(chain: Chain, options: Option<ChainOptions>) -> Self {
    let options = match options.clone() {
      Some(options) => options,
      None => ChainOptions::default(),
    };

    Self {
      chain,
      options,
      cached_chain_info: None,
    }
  }

  async fn chain_info(&mut self) -> Result<ChainInfo> {
    if self.options().is_cache() {
      match &self.cached_chain_info {
        Some(info) => Ok(info.clone()),
        None => {
          let info = self.chain.info().await?;
          self.cached_chain_info = Some(info.clone());
          Ok(info)
        }
      }
    } else {
      Ok(self.chain.info().await?)
    }
  }

  fn beacon_url(&self, round: String) -> Result<String> {
    let query = match self.options().is_cache() {
      true => format!("?{:#?}", time::Instant::now()),
      false => String::from(""),
    };
    Ok(format!("{}/public/{round}{query}", self.chain.base_url()))
  }

  async fn verify_beacon(&mut self, beacon: RandomnessBeacon) -> Result<RandomnessBeacon> {
    if !self.options().is_beacon_verification() {
      return Ok(beacon);
    }

    match beacon.verify(self.chain_info().await?)? {
      true => Ok(beacon),
      false => Err(anyhow!("Beacon does not validate")),
    }
  }
}

#[async_trait]
impl ChainClient for HttpChainClient {
    fn options(&self) -> ChainOptions  {
        self.options.clone()
    }

    async fn latest(&mut self) -> Result<RandomnessBeacon> {
      let beacon = reqwest::get(self.beacon_url(String::from("latest"))?).await?.json::<RandomnessBeacon>().await?;

      self.verify_beacon(beacon).await
    }

    async fn get(&mut self, round_number: u64) -> Result<RandomnessBeacon> {
      let beacon = reqwest::get(self.beacon_url(round_number.to_string())?).await?.json::<RandomnessBeacon>().await?;

      self.verify_beacon(beacon).await
    }

    fn chain(&self) -> Chain {
        self.chain.clone()
    }
}
