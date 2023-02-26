use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::beacon::RandomnessBeacon;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Additional information about the chain.
pub struct ChainMetadata {
    #[serde(rename(serialize = "beaconID", deserialize = "beaconID"))]
    beacon_id: String,
}

impl ChainMetadata {
    pub fn new(beacon_id: String) -> Self {
        Self { beacon_id }
    }

    /// The ID of the beacon chain this `ChainInfo` corresponds to.
    pub fn beacon_id(&self) -> String {
        self.beacon_id.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainInfo {
    public_key: String,
    period: u64,
    genesis_time: u64,
    hash: String,
    #[serde(rename(serialize = "groupHash", deserialize = "groupHash"))]
    group_hash: String,
    #[serde(rename(serialize = "schemeID", deserialize = "schemeID"))]
    scheme_id: String,
    metadata: ChainMetadata,
}

impl ChainInfo {
    /// Hex encoded BLS12-381 public key.
    pub fn public_key(&self) -> String {
        self.public_key.clone()
    }

    /// How often the network emits randomness (in seconds).
    pub fn period(&self) -> u64 {
        self.period
    }

    /// Time of the round 0 of the network (in epoch seconds).
    pub fn genesis_time(&self) -> u64 {
        self.genesis_time
    }

    /// Hash identifying this specific chain of beacons.
    pub fn hash(&self) -> String {
        self.hash.clone()
    }

    /// A hash of the group file containing details of all the nodes participating in the network.
    pub fn group_hash(&self) -> String {
        self.group_hash.clone()
    }

    /// The version/format of cryptography.
    pub fn scheme_id(&self) -> String {
        self.scheme_id.clone()
    }

    /// Additional information about the chain.
    pub fn metadata(&self) -> ChainMetadata {
        self.metadata.clone()
    }
}

#[derive(Debug, Clone)]
/// HTTP drand chain, identified by a base URL
/// e.g https://drand.cloudflare.com
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
/// Retrieval and validation options when interacting with a chain.
/// This controls beacons validation, chain validation, and cache on retrieval.
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
/// Parameters that can be used to validate a chain is the expected one.
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
/// Drand client, that can retrieve and validate information from a given chain.
pub trait ChainClient {
    /// Options that are used to validate chain result.
    fn options(&self) -> ChainOptions;
    /// Retrieve latest beacon.
    /// This is retrieved and validated based on the client options.
    async fn latest(&self) -> Result<RandomnessBeacon>;
    /// Retrieve specific round beacon.
    /// This is retrieved and validated based on the client options.
    async fn get(&self, round_number: u64) -> Result<RandomnessBeacon>;
    /// Chain the client is associated to.
    fn chain(&self) -> Chain;
}
