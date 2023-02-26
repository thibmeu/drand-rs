use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Mutex;
use std::time;

use crate::{
    beacon::RandomnessBeacon,
    chain::{Chain, ChainClient, ChainInfo, ChainOptions},
};

/// HTTP Client for drand
/// Queries a specified HTTP endpoint given by `chain`, with specific `options`
/// By default, the client verifies answers, and caches retrieved chain informations
pub struct HttpChainClient {
    chain: Chain,
    options: ChainOptions,
    cached_chain_info: Mutex<Option<ChainInfo>>,
}

impl HttpChainClient {
    pub fn new(chain: Chain, options: Option<ChainOptions>) -> Self {
        let options = match options {
            Some(options) => options,
            None => ChainOptions::default(),
        };

        Self {
            chain,
            options,
            cached_chain_info: Mutex::new(None),
        }
    }

    async fn chain_info(&self) -> Result<ChainInfo> {
        if self.options().is_cache() {
            let cached = self.cached_chain_info.lock().unwrap().to_owned();
            match cached {
                Some(info) => Ok(info),
                None => {
                    let info = self.chain.info().await?;
                    if !self.options().verify(info.clone()) {
                        return Err(anyhow!("Chain info is invalid"));
                    }
                    *self.cached_chain_info.lock().unwrap() = Some(info.clone());
                    Ok(info)
                }
            }
        } else {
            let info = self.chain.info().await?;
            if !self.options().verify(info.clone()) {
                return Err(anyhow!("Chain info is invalid"));
            }
            Ok(info)
        }
    }

    fn beacon_url(&self, round: String) -> Result<String> {
        let query = match self.options().is_cache() {
            true => format!(
                "?{}",
                time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)?
                    .as_millis()
            ),
            false => String::from(""),
        };
        Ok(format!("{}/public/{round}{query}", self.chain.base_url()))
    }

    async fn verify_beacon(&self, beacon: RandomnessBeacon) -> Result<RandomnessBeacon> {
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
    fn options(&self) -> ChainOptions {
        self.options.clone()
    }

    async fn latest(&self) -> Result<RandomnessBeacon> {
        let beacon = reqwest::get(self.beacon_url(String::from("latest"))?)
            .await?
            .json::<RandomnessBeacon>()
            .await?;

        self.verify_beacon(beacon).await
    }

    async fn get(&self, round_number: u64) -> Result<RandomnessBeacon> {
        let beacon = reqwest::get(self.beacon_url(round_number.to_string())?)
            .await?
            .json::<RandomnessBeacon>()
            .await?;

        self.verify_beacon(beacon).await
    }

    fn chain(&self) -> Chain {
        self.chain.clone()
    }
}
