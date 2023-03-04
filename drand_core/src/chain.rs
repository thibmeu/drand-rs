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
    #[serde(with = "hex::serde")]
    public_key: Vec<u8>,
    period: u64,
    genesis_time: u64,
    #[serde(with = "hex::serde")]
    hash: Vec<u8>,
    #[serde(
        rename(serialize = "groupHash", deserialize = "groupHash"),
        with = "hex::serde"
    )]
    group_hash: Vec<u8>,
    #[serde(rename(serialize = "schemeID", deserialize = "schemeID"))]
    scheme_id: String,
    metadata: ChainMetadata,
}

impl ChainInfo {
    /// Hex encoded BLS12-381 public key.
    pub fn public_key(&self) -> Vec<u8> {
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
    pub fn hash(&self) -> Vec<u8> {
        self.hash.clone()
    }

    /// A hash of the group file containing details of all the nodes participating in the network.
    pub fn group_hash(&self) -> Vec<u8> {
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
    hash: Option<Vec<u8>>,
    public_key: Option<Vec<u8>>,
}

impl ChainVerification {
    pub fn new(hash: Option<Vec<u8>>, public_key: Option<Vec<u8>>) -> Self {
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

#[cfg(test)]
pub mod tests {
    use super::*;

    /// drand mainnet (curl -sS https://drand.cloudflare.com/info)
    pub fn chained_chain_info() -> ChainInfo {
        serde_json::from_str(r#"{
            "public_key": "868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31",
            "period": 30,
            "genesis_time": 1595431050,
            "hash": "8990e7a9aaed2ffed73dbd7092123d6f289930540d7651336225dc172e51b2ce",
            "groupHash": "176f93498eac9ca337150b46d21dd58673ea4e3581185f869672e59fa4cb390a",
            "schemeID": "pedersen-bls-chained",
            "metadata": {
                "beaconID": "default"
            }
        }"#).unwrap()
    }

    /// drand testnet (curl -sS https://pl-us.testnet.drand.sh/7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf/info)
    pub fn unchained_chain_info() -> ChainInfo {
        serde_json::from_str(r#"{
            "public_key": "8200fc249deb0148eb918d6e213980c5d01acd7fc251900d9260136da3b54836ce125172399ddc69c4e3e11429b62c11",
            "period": 3,
            "genesis_time": 1651677099,
            "hash": "7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf",
            "groupHash": "65083634d852ae169e21b6ce5f0410be9ed4cc679b9970236f7875cff667e13d",
            "schemeID": "pedersen-bls-unchained",
            "metadata": {
                "beaconID": "testnet-unchained-3s"
            }
        }"#).unwrap()
    }

    /// drand testnet (curl -sS https://testnet0-api.drand.cloudflare.com/f3827d772c155f95a9fda8901ddd59591a082df5ac6efe3a479ddb1f5eeb202c/info)
    pub fn unchained_chain_on_g1_info() -> ChainInfo {
        serde_json::from_str(r#"{
            "public_key": "8f6e58c3dbc6d7e58e32baee6881fecc854161b4227c40b01ae7f0593cea964599648f91a0fa2d6b489a7fb0a552b959014007e05d0c069991be4d064bbe28275bd4c3a3cabf16c48f86f4566909dd6eb6d0e84fd6069c414562ca6abf5fdc13",
            "period": 3,
            "genesis_time": 1675262550,
            "hash": "f3827d772c155f95a9fda8901ddd59591a082df5ac6efe3a479ddb1f5eeb202c",
            "groupHash": "73c191da8ca22628987bc9fb330e2b82f9e38728a8708b10b42b43c90643b798",
            "schemeID": "bls-unchained-on-g1",
            "metadata": {
                "beaconID": "testnet-g"
            }
        }"#).unwrap()
    }

    #[test]
    fn chain_verification_success_works() {
        // Full validation should pass
        let full_verification = ChainVerification::new(
            Some(chained_chain_info().hash()),
            Some(chained_chain_info().public_key()),
        );
        assert!(full_verification.verify(chained_chain_info()));

        // Validate only the hash
        let hash_verification = ChainVerification::new(Some(chained_chain_info().hash()), None);
        assert!(hash_verification.verify(chained_chain_info()));
        let hash_verification = ChainVerification::new(Some(chained_chain_info().hash()), None);
        let mut chain_info = chained_chain_info();
        chain_info.public_key = unchained_chain_info().public_key();
        assert!(hash_verification.verify(chain_info));

        // Validate only the public key
        let public_key_verification =
            ChainVerification::new(None, Some(chained_chain_info().public_key()));
        assert!(public_key_verification.verify(chained_chain_info()));
        let mut chain_info = chained_chain_info();
        chain_info.hash = unchained_chain_info().hash();
        assert!(public_key_verification.verify(chain_info));

        // Don't validate
        let no_verification = ChainVerification::new(None, None);
        assert!(no_verification.verify(chained_chain_info()));
    }

    #[test]
    fn chain_verification_failure_works() {
        // Full validation should fail when public key is invalid
        let full_verification = ChainVerification::new(
            Some(chained_chain_info().hash()),
            Some(unchained_chain_info().public_key()),
        );
        assert!(!full_verification.verify(chained_chain_info()));
        // Full validation should fail when hash is invalid
        let full_verification = ChainVerification::new(
            Some(unchained_chain_info().hash()),
            Some(chained_chain_info().public_key()),
        );
        assert!(!full_verification.verify(chained_chain_info()));

        // Validate only the hash works with invalid public key
        let hash_verification = ChainVerification::new(Some(unchained_chain_info().hash()), None);
        assert!(!hash_verification.verify(chained_chain_info()));

        // Validate only the public key
        let public_key_verification =
            ChainVerification::new(None, Some(unchained_chain_info().public_key()));
        assert!(!public_key_verification.verify(chained_chain_info()));
    }

    impl PartialEq for ChainMetadata {
        fn eq(&self, other: &Self) -> bool {
            self.beacon_id == other.beacon_id
        }
    }

    impl PartialEq for ChainInfo {
        fn eq(&self, other: &Self) -> bool {
            self.public_key == other.public_key
                && self.period == other.period
                && self.genesis_time == other.genesis_time
                && self.hash == other.hash
                && self.group_hash == other.group_hash
                && self.scheme_id == other.scheme_id
                && self.metadata == other.metadata
        }
    }

    #[tokio::test]
    async fn chain_info_retrieval_success_works() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/info")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_chain_info()).unwrap())
            .create_async()
            .await;

        let chain = Chain::new(server.url().as_str());
        let chain_info = chain.info().await;
        match chain_info {
            Ok(info) => assert_eq!(info, chained_chain_info()),
            Err(_err) => panic!(""),
        }
    }

    #[tokio::test]
    async fn chain_info_retrieval_failure_works() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/info")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"not_chain_info": true}"#)
            .create_async()
            .await;

        let chain = Chain::new(server.url().as_str());
        let chain_info = chain.info().await;
        match chain_info {
            Ok(_) => panic!(""),
            Err(_err) => (),
        }
    }
}
