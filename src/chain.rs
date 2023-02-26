use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::beacon::RandomnessBeacon;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainMetadata {
    #[serde(rename(serialize = "beaconID", deserialize = "beaconID"))]
    beacon_id: String, // the ID of the beacon chain this `ChainInfo` corresponds to
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainInfo {
    public_key: String, // hex encoded BLS12-381 public key
    period: u64,        // how often the network emits randomness (in seconds)
    genesis_time: u64,  // the time of the round 0 of the network (in epoch seconds)
    hash: String,       // the hash identifying this specific chain of beacons
    #[serde(rename(serialize = "groupHash", deserialize = "groupHash"))]
    group_hash: String, // a hash of the group file containing details of all the nodes participating in the network
    #[serde(rename(serialize = "schemeID", deserialize = "schemeID"))]
    scheme_id: String, // the version/format of cryptography
    metadata: ChainMetadata,
}

impl ChainInfo {
    pub fn public_key(&self) -> String {
        self.public_key.clone()
    }

    pub fn period(&self) -> u64 {
        self.period
    }

    pub fn genesis_time(&self) -> u64 {
        self.genesis_time
    }

    pub fn hash(&self) -> String {
        self.hash.clone()
    }

    pub fn group_hash(&self) -> String {
        self.group_hash.clone()
    }

    pub fn scheme_id(&self) -> String {
        self.scheme_id.clone()
    }

    pub fn metadata(&self) -> ChainMetadata {
        self.metadata.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Chain {
    base_url: String,
}

impl Chain {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: String::from(base_url),
        }
    }

    pub fn base_url(&self) -> String {
        self.base_url.clone()
    }

    pub async fn info(&self) -> Result<ChainInfo> {
        Ok(reqwest::get(format!("{}/info", self.base_url))
            .await?
            .json::<ChainInfo>()
            .await?)
    }
}

#[derive(Debug, Clone)]
pub struct ChainOptions {
    is_beacon_verification: bool,
    is_cache: bool,
    chain_verification: ChainVerification,
}

impl ChainOptions {
    pub fn new(
        is_beacon_verification: bool,
        is_cache: bool,
        chain_verification: Option<ChainVerification>,
    ) -> Self {
        let chain_verification = match chain_verification {
            Some(cv) => cv,
            None => ChainVerification::default(),
        };
        Self {
            is_beacon_verification,
            is_cache,
            chain_verification,
        }
    }

    pub fn is_beacon_verification(&self) -> bool {
        self.is_beacon_verification
    }

    pub fn is_cache(&self) -> bool {
        self.is_cache
    }

    pub fn verify(&self, info: ChainInfo) -> bool {
        self.chain_verification.verify(info)
    }
}

impl Default for ChainOptions {
    fn default() -> Self {
        Self::new(true, true, None)
    }
}

#[derive(Debug, Clone)]
pub struct ChainVerification {
    hash: Option<String>,
    public_key: Option<String>,
}

impl ChainVerification {
    pub fn new(hash: Option<String>, public_key: Option<String>) -> Self {
        Self { hash, public_key }
    }

    pub fn verify(&self, info: ChainInfo) -> bool {
        let ok_hash = match &self.hash {
            Some(h) => info.hash == *h,
            None => true,
        };
        let ok_public_key = match &self.public_key {
            Some(pk) => info.public_key == *pk,
            None => true,
        };
        ok_hash && ok_public_key
    }
}

impl Default for ChainVerification {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[async_trait]
pub trait ChainClient {
    fn options(&self) -> ChainOptions;
    async fn latest(&self) -> Result<RandomnessBeacon>;
    async fn get(&self, round_number: u64) -> Result<RandomnessBeacon>;
    fn chain(&self) -> Chain;
}
