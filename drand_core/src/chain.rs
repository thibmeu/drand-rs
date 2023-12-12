use serde::{Deserialize, Serialize};

use crate::{beacon::RandomnessBeacon, Result};

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

impl PartialEq for ChainMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.beacon_id == other.beacon_id
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

    /// Time of the round 1 of the network (in epoch seconds).
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

    /// Is the chain relying on RFC 9380 Hashing to elliptic curves
    pub fn is_rfc9380(&self) -> bool {
        self.scheme_id.contains("rfc9380")
    }

    pub fn is_unchained(&self) -> bool {
        self.scheme_id.contains("unchained")
    }

    /// Additional information about the chain.
    pub fn metadata(&self) -> ChainMetadata {
        self.metadata.clone()
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
        Self {
            is_beacon_verification,
            is_cache,
            chain_verification: chain_verification.unwrap_or_default(),
        }
    }

    pub fn is_beacon_verification(&self) -> bool {
        self.is_beacon_verification
    }

    pub fn is_cache(&self) -> bool {
        self.is_cache
    }

    pub fn verify(&self, info: &ChainInfo) -> bool {
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

    pub fn verify(&self, info: &ChainInfo) -> bool {
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

impl From<ChainInfo> for ChainVerification {
    fn from(info: ChainInfo) -> Self {
        Self::new(Some(info.hash()), Some(info.public_key()))
    }
}

/// Drand client, that can retrieve and validate information from a given chain.
pub trait ChainClient {
    /// Options that are used to validate chain result.
    fn options(&self) -> ChainOptions;
    /// Retrieve latest beacon.
    /// This is retrieved and validated based on the client options.
    fn latest(&self) -> Result<RandomnessBeacon>;
    /// Retrieve specific round beacon.
    /// This is retrieved and validated based on the client options.
    fn get(&self, round_number: u64) -> Result<RandomnessBeacon>;
    /// Chain info the client is associated to.
    fn chain_info(&self) -> Result<ChainInfo>;
}

#[cfg(feature = "time")]
#[derive(Debug, Serialize, Deserialize)]
/// Time information for a chain.
/// Genesis and period, allowing to reconstruct time information of a given beacon.
pub struct ChainTimeInfo {
    genesis_time: u64,
    period: u64,
}

#[cfg(feature = "time")]
impl ChainTimeInfo {
    pub fn new(genesis_time: u64, period: u64) -> Self {
        Self {
            genesis_time,
            period,
        }
    }

    pub fn genesis_time(&self) -> u64 {
        self.genesis_time
    }

    pub fn period(&self) -> u64 {
        self.period
    }
}

#[cfg(test)]
pub mod tests {
    use serde_json::json;
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

    /// drand fastnet (curl -sS https://drand.cloudflare.com/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/info)
    pub fn unchained_chain_on_g1_info() -> ChainInfo {
        serde_json::from_str(r#"{
            "public_key": "a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e",
            "period": 3,
            "genesis_time": 1677685200,
            "hash": "dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493",
            "groupHash": "a81e9d63f614ccdb144b8ff79fbd4d5a2d22055c0bfe4ee9a8092003dab1c6c0",
            "schemeID": "bls-unchained-on-g1",
            "metadata": {
              "beaconID": "fastnet"
            }
        }"#).unwrap()
    }

    /// From drand Slack https://drandworkspace.slack.com/archives/C02FWA217GF/p1686583505902169
    pub fn unchained_chain_on_g1_rfc_info() -> ChainInfo {
        serde_json::from_str(r#"{
            "public_key": "a1ee12542360bf75742bcade13d6134e7d5283d9eb782887c47d3d9725f05805d37b0106b7f744395bf82c175dd7434a169e998f188a657a030d588892c0cd2c01f996aaf331c4d8bc5b9734bbe261d09e7d2d39ef88b635077f262bd7bbb30f",
            "period": 3,
            "genesis_time": 1677685200,
            "hash": "dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493",
            "groupHash": "a81e9d63f614ccdb144b8ff79fbd4d5a2d22055c0bfe4ee9a8092003dab1c6c0",
            "schemeID": "bls-unchained-g1-rfc9380",
            "metadata": {
              "beaconID": "does-not-exist-slacn"
            }
        }"#).unwrap()
    }

    pub fn create_chained_info_with_genesis(genesis_time: u64) -> ChainInfo {
        serde_json::from_value(json!({
                        "public_key": "868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31",
            "period": 30,
            "genesis_time": genesis_time,
            "hash": "8990e7a9aaed2ffed73dbd7092123d6f289930540d7651336225dc172e51b2ce",
            "groupHash": "176f93498eac9ca337150b46d21dd58673ea4e3581185f869672e59fa4cb390a",
            "schemeID": "pedersen-bls-chained",
            "metadata": {
                "beaconID": "default"
            }
        })).unwrap()
    }

    pub fn create_unchained_info_with_genesis(genesis_time: u64) -> ChainInfo {
        serde_json::from_value(json!({
            "public_key": "8200fc249deb0148eb918d6e213980c5d01acd7fc251900d9260136da3b54836ce125172399ddc69c4e3e11429b62c11",
            "period": 3,
            "genesis_time": genesis_time,
            "hash": "7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf",
            "groupHash": "65083634d852ae169e21b6ce5f0410be9ed4cc679b9970236f7875cff667e13d",
            "schemeID": "pedersen-bls-unchained",
            "metadata": {
                "beaconID": "testnet-unchained-3s"
            }
        })).unwrap()
    }

    #[test]
    fn chain_verification_success_works() {
        // Full validation should pass
        let full_verification = ChainVerification::new(
            Some(chained_chain_info().hash()),
            Some(chained_chain_info().public_key()),
        );
        assert!(full_verification.verify(&chained_chain_info()));

        // Validate only the hash
        let hash_verification = ChainVerification::new(Some(chained_chain_info().hash()), None);
        assert!(hash_verification.verify(&chained_chain_info()));
        let hash_verification = ChainVerification::new(Some(chained_chain_info().hash()), None);
        let mut chain_info = chained_chain_info();
        chain_info.public_key = unchained_chain_info().public_key();
        assert!(hash_verification.verify(&chain_info));

        // Validate only the public key
        let public_key_verification =
            ChainVerification::new(None, Some(chained_chain_info().public_key()));
        assert!(public_key_verification.verify(&chained_chain_info()));
        let mut chain_info = chained_chain_info();
        chain_info.hash = unchained_chain_info().hash();
        assert!(public_key_verification.verify(&chain_info));

        // Don't validate
        let no_verification = ChainVerification::new(None, None);
        assert!(no_verification.verify(&chained_chain_info()));
    }

    #[test]
    fn chain_verification_failure_works() {
        // Full validation should fail when public key is invalid
        let full_verification = ChainVerification::new(
            Some(chained_chain_info().hash()),
            Some(unchained_chain_info().public_key()),
        );
        assert!(!full_verification.verify(&chained_chain_info()));
        // Full validation should fail when hash is invalid
        let full_verification = ChainVerification::new(
            Some(unchained_chain_info().hash()),
            Some(chained_chain_info().public_key()),
        );
        assert!(!full_verification.verify(&chained_chain_info()));

        // Validate only the hash works with invalid public key
        let hash_verification = ChainVerification::new(Some(unchained_chain_info().hash()), None);
        assert!(!hash_verification.verify(&chained_chain_info()));

        // Validate only the public key
        let public_key_verification =
            ChainVerification::new(None, Some(unchained_chain_info().public_key()));
        assert!(!public_key_verification.verify(&chained_chain_info()));
    }
}
