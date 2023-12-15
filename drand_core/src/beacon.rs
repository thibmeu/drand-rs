use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
#[cfg(feature = "time")]
use time::{
    ext::NumericalDuration, format_description::well_known::Rfc3339, Duration, OffsetDateTime,
};

#[cfg(feature = "time")]
use crate::chain::ChainTimeInfo;
use crate::{chain::ChainInfo, DrandError, Result};

#[derive(Error, Debug)]
pub enum BeaconError {
    #[cfg(feature = "time")]
    #[error("cannot parse duration")]
    DurationParse,
    #[error("beacon not found")]
    NotFound,
    #[error("parsing failed")]
    Parsing,
    #[error("round mismatch")]
    RoundMismatch,
    #[error("validation failed")]
    Validation,
}

#[derive(Clone, Debug, Serialize)]
pub struct RandomnessBeacon {
    #[serde(flatten)]
    beacon: ApiBeacon,
    #[serde(skip_serializing)]
    time: u64,
}

impl RandomnessBeacon {
    pub(crate) fn new(beacon: ApiBeacon, time: u64) -> Self {
        Self { beacon, time }
    }

    pub fn verify(&self, info: ChainInfo) -> Result<bool> {
        self.beacon.verify(info)
    }

    pub fn round(&self) -> u64 {
        self.beacon.round()
    }

    pub fn randomness(&self) -> Vec<u8> {
        self.beacon.randomness()
    }

    pub fn is_unchained(&self) -> bool {
        self.beacon.is_unchained()
    }

    pub fn signature(&self) -> Vec<u8> {
        self.beacon.signature()
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    #[cfg(test)]
    pub(crate) fn beacon(&self) -> ApiBeacon {
        self.beacon.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
/// Random beacon as generated by drand.
/// These can be chained or unchained, and should be verifiable against a chain.
pub enum ApiBeacon {
    ChainedBeacon(ChainedBeacon),
    UnchainedBeacon(UnchainedBeacon),
}

impl ApiBeacon {
    pub fn verify(&self, info: ChainInfo) -> Result<bool> {
        if self.is_unchained() != info.is_unchained()
            || self.is_g1() && !info.scheme_id().contains("g1")
        {
            return Ok(false);
        }

        let signature_verify = crate::bls_signatures::verify(
            self.dst(&info),
            &self.signature(),
            &self.message()?,
            &info.public_key(),
        )?;

        let mut hasher = Sha256::new();
        hasher.update(self.signature());
        let randomness = hasher.finalize().to_vec();
        let randomness_verify = randomness == self.randomness();

        Ok(signature_verify && randomness_verify)
    }

    pub fn round(&self) -> u64 {
        match self {
            Self::ChainedBeacon(chained) => chained.round,
            Self::UnchainedBeacon(unchained) => unchained.round,
        }
    }

    pub fn randomness(&self) -> Vec<u8> {
        match self {
            Self::ChainedBeacon(chained) => chained.randomness.clone(),
            Self::UnchainedBeacon(unchained) => unchained.randomness.clone(),
        }
    }

    fn dst(&self, info: &ChainInfo) -> &[u8] {
        // Name of the HashToCurve RFC compliant scheme has been decided upon in https://github.com/drand/drand/pull/1249
        if info.is_rfc9380() && info.scheme_id().contains("g1") {
            crate::bls_signatures::G1_DOMAIN
        } else {
            crate::bls_signatures::G2_DOMAIN
        }
    }

    pub fn is_unchained(&self) -> bool {
        match self {
            Self::ChainedBeacon(_) => false,
            Self::UnchainedBeacon(_) => true,
        }
    }

    fn is_g1(&self) -> bool {
        match self {
            Self::ChainedBeacon(_) => false,
            Self::UnchainedBeacon(unchained) => unchained.signature.len() == 48,
        }
    }

    pub fn signature(&self) -> Vec<u8> {
        match self {
            Self::ChainedBeacon(chained) => chained.signature.clone(),
            Self::UnchainedBeacon(unchained) => unchained.signature.clone(),
        }
    }
}

impl Message for ApiBeacon {
    fn message(&self) -> Result<Vec<u8>> {
        match self {
            Self::ChainedBeacon(chained) => chained.message(),
            Self::UnchainedBeacon(unchained) => unchained.message(),
        }
    }
}

impl From<ChainedBeacon> for ApiBeacon {
    fn from(b: ChainedBeacon) -> Self {
        Self::ChainedBeacon(b)
    }
}

impl From<UnchainedBeacon> for ApiBeacon {
    fn from(b: UnchainedBeacon) -> Self {
        Self::UnchainedBeacon(b)
    }
}

/// Package item to be validated against a BLS signature given a public key.
trait Message {
    fn message(&self) -> Result<Vec<u8>>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Chained drand beacon.
/// Each signature depends on the previous one, as well as on the round.
pub struct ChainedBeacon {
    round: u64,
    #[serde(with = "hex::serde")]
    randomness: Vec<u8>,
    #[serde(with = "hex::serde")]
    signature: Vec<u8>,
    #[serde(with = "hex::serde")]
    previous_signature: Vec<u8>,
}

impl Message for ChainedBeacon {
    fn message(&self) -> Result<Vec<u8>> {
        // First round signature is on the genesis seed, which size is 32B, and not 96B like G2 signatures.
        let len = if self.round == 1 { 32 } else { 96 };
        let mut buf = vec![0; len + 8];
        let (signature_buf, round_buf) = buf.split_at_mut(len);

        signature_buf.clone_from_slice(&self.previous_signature);
        round_buf.clone_from_slice(&self.round.to_be_bytes());

        let mut hasher = Sha256::new();
        hasher.update(buf);
        Ok(hasher.finalize().to_vec())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Unchained drand beacon.
/// Each signature only depends on the round number.
pub struct UnchainedBeacon {
    round: u64,
    #[serde(with = "hex::serde")]
    randomness: Vec<u8>,
    #[serde(with = "hex::serde")]
    signature: Vec<u8>,
}

impl Message for UnchainedBeacon {
    fn message(&self) -> Result<Vec<u8>> {
        let buf = self.round.to_be_bytes();

        let mut hasher = Sha256::new();
        hasher.update(buf);
        Ok(hasher.finalize().to_vec())
    }
}

#[cfg(feature = "time")]
impl From<ChainInfo> for ChainTimeInfo {
    fn from(value: ChainInfo) -> Self {
        Self::new(value.genesis_time(), value.period())
    }
}

#[cfg(feature = "time")]
#[derive(Debug, Serialize, Deserialize)]
/// Time of a randomness beacon as seen by drand.
/// Round and absolute are uniquely tied to a round.
/// Relative time is generated upon object creation.
pub struct RandomnessBeaconTime {
    round: u64,
    relative: Duration,
    #[cfg_attr(feature = "time", serde(with = "time::serde::rfc3339"))]
    absolute: OffsetDateTime,
}

#[cfg(feature = "time")]
impl RandomnessBeaconTime {
    /// round can be:
    /// * a specific round. e.g. 123,
    /// * a duration. e.g. 30s,
    /// * an RFC3339 date. e.g. 2023-06-28 21:30:22
    pub fn new(info: &ChainTimeInfo, round: &str) -> Self {
        match (
            round.parse::<u64>(),
            Self::parse_duration(round),
            OffsetDateTime::parse(round, &Rfc3339),
        ) {
            (Ok(round), Err(_), Err(_)) => Self::from_round(info, round),
            (Err(_), Ok(relative), Err(_)) => Self::from_duration(info, relative),
            (Err(_), Err(_), Ok(absolute)) => Self::from_datetime(info, absolute),
            _ => unreachable!(),
        }
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn relative(&self) -> Duration {
        self.relative
    }

    pub fn absolute(&self) -> OffsetDateTime {
        self.absolute
    }

    pub fn from_round(info: &ChainTimeInfo, round: u64) -> Self {
        let genesis = OffsetDateTime::from_unix_timestamp(info.genesis_time() as i64).unwrap();

        let absolute = genesis + (((round - 1) * info.period()) as i64).seconds();
        let relative = absolute - OffsetDateTime::now_utc();
        Self {
            round,
            relative,
            absolute,
        }
    }

    fn from_duration(info: &ChainTimeInfo, relative: Duration) -> Self {
        let genesis = OffsetDateTime::from_unix_timestamp(info.genesis_time() as i64).unwrap();

        let absolute = OffsetDateTime::now_utc() + relative;
        let round = ((absolute - genesis).whole_seconds() / (info.period() as i64) + 1) as u64;

        Self {
            round,
            relative,
            absolute,
        }
    }

    fn from_datetime(info: &ChainTimeInfo, absolute: OffsetDateTime) -> Self {
        let genesis = OffsetDateTime::from_unix_timestamp(info.genesis_time() as i64).unwrap();

        let relative = absolute - OffsetDateTime::now_utc();
        let round = ((absolute - genesis).whole_seconds() / (info.period() as i64) + 1) as u64;

        Self {
            round,
            relative,
            absolute,
        }
    }

    fn parse_duration(duration: &str) -> Result<Duration> {
        let l = duration.len() - 1;
        let principal = duration[0..l]
            .parse::<i64>()
            .map_err(|_| -> DrandError { Box::new(BeaconError::DurationParse).into() })?;

        let duration = match duration.chars().last().unwrap() {
            's' => principal.seconds(),
            'm' => principal.minutes(),
            'h' => principal.hours(),
            'd' => principal.days(),
            _char => return Err(Box::new(BeaconError::DurationParse).into()),
        };
        Ok(duration)
    }
}

#[cfg(test)]
pub mod tests {
    use std::ops::Sub;

    use crate::chain::{
        tests::chained_chain_info,
        tests::{unchained_chain_info, unchained_chain_on_g1_info, unchained_chain_on_g1_rfc_info},
    };

    use super::*;

    /// drand mainnet (curl -sS https://drand.cloudflare.com/public/1000000)
    pub fn chained_beacon() -> ApiBeacon {
        serde_json::from_str(r#"{
            "round": 1000000,
            "randomness": "a26ba4d229c666f52a06f1a9be1278dcc7a80dbc1dd2004a1ae7b63cb79fd37e",
            "signature": "87e355169c4410a8ad6d3e7f5094b2122932c1062f603e6628aba2e4cb54f46c3bf1083c3537cd3b99e8296784f46fb40e090961cf9634f02c7dc2a96b69fc3c03735bc419962780a71245b72f81882cf6bb9c961bcf32da5624993bb747c9e5",
            "previous_signature": "86bbc40c9d9347568967add4ddf6e351aff604352a7e1eec9b20dea4ca531ed6c7d38de9956ffc3bb5a7fabe28b3a36b069c8113bd9824135c3bff9b03359476f6b03beec179d4aeff456f4d34bbf702b9af78c3bb44e1892ace8e581bf4afa9"
        }"#).unwrap()
    }

    /// drand mainnet (curl -sS https://drand.cloudflare.com/public/1)
    pub fn chained_beacon_1() -> ApiBeacon {
        serde_json::from_str(r#"{
            "round": 1,
            "randomness": "101297f1ca7dc44ef6088d94ad5fb7ba03455dc33d53ddb412bbc4564ed986ec",
            "signature": "8d61d9100567de44682506aea1a7a6fa6e5491cd27a0a0ed349ef6910ac5ac20ff7bc3e09d7c046566c9f7f3c6f3b10104990e7cb424998203d8f7de586fb7fa5f60045417a432684f85093b06ca91c769f0e7ca19268375e659c2a2352b4655",
            "previous_signature": "176f93498eac9ca337150b46d21dd58673ea4e3581185f869672e59fa4cb390a"
          }"#).unwrap()
    }

    /// drand testnet (curl -sS https://pl-us.testnet.drand.sh/7672797f548f3f4748ac4bf3352fc6c6b6468c9ad40ad456a397545c6e2df5bf/public/1000000)
    pub fn unchained_beacon() -> ApiBeacon {
        serde_json::from_str(r#"{
            "round": 1000000,
            "randomness": "6671747f7d838f18159c474579ea19e8d863e8c25e5271fd7f18ca2ac85181cf",
            "signature": "86b265e10e060805d20dca88f70f6b5e62d5956e7790d32029dfb73fbcd1996bc7aebdea7aeaf74dac0ca2b3ce8f7a6a0399f224a05fe740c0bac9da638212082b0ed21b1a8c5e44a33123f28955ef0713e93e21f6af0cda4073d9a73387434d"
        }"#).unwrap()
    }

    /// drand fastnet (curl -sS https://drand.cloudflare.com/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/100000)
    pub fn unchained_beacon_on_g1() -> ApiBeacon {
        serde_json::from_str(r#"{
            "round": 100000,
            "randomness": "37aa25aa1e0b52440502e6f841c956bf72d693770a511e59768ecb7777c172ce",
            "signature": "b370f411d5479fc342b504347226e4b543fee28698fa721876d55d36c12a20f3f49b7abd31ee99979e2d28e14f1d3152"
        }"#).unwrap()
    }

    /// From drand Slack https://drandworkspace.slack.com/archives/C02FWA217GF/p1686583505902169
    pub fn unchained_beacon_on_g1_rfc() -> ApiBeacon {
        serde_json::from_str(r#"{
            "round": 3,
            "randomness":"9e9829dfb34bd8db3e21c28e13aefecd86e007ebd19d6bb8a5cee99c0a34798f",
            "signature":"b98dae74f6a9d2ec79d75ba273dcfda86a45d589412860eb4c0fd056b00654dbf667c1b6884987c9aee0d43f8ba9db52"
        }"#).unwrap()
    }

    /// invalid beacon. Round should be 1,000,000, but it 1
    pub fn invalid_beacon() -> ApiBeacon {
        serde_json::from_str(r#"{
            "round": 1234,
            "randomness": "a26ba4d229c666f52a06f1a9be1278dcc7a80dbc1dd2004a1ae7b63cb79fd37e",
            "signature": "87e355169c4410a8ad6d3e7f5094b2122932c1062f603e6628aba2e4cb54f46c3bf1083c3537cd3b99e8296784f46fb40e090961cf9634f02c7dc2a96b69fc3c03735bc419962780a71245b72f81882cf6bb9c961bcf32da5624993bb747c9e5",
            "previous_signature": "86bbc40c9d9347568967add4ddf6e351aff604352a7e1eec9b20dea4ca531ed6c7d38de9956ffc3bb5a7fabe28b3a36b069c8113bd9824135c3bff9b03359476f6b03beec179d4aeff456f4d34bbf702b9af78c3bb44e1892ace8e581bf4afa9"
        }"#).unwrap()
    }

    impl PartialEq for ApiBeacon {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::ChainedBeacon(chained), Self::ChainedBeacon(other)) => chained == other,
                (Self::UnchainedBeacon(unchained), Self::UnchainedBeacon(other)) => {
                    unchained == other
                }
                _ => false,
            }
        }
    }

    impl PartialEq for ChainedBeacon {
        fn eq(&self, other: &Self) -> bool {
            self.randomness == other.randomness
                && self.round == other.round
                && self.signature == other.signature
                && self.previous_signature == other.previous_signature
        }
    }

    impl PartialEq for UnchainedBeacon {
        fn eq(&self, other: &Self) -> bool {
            self.randomness == other.randomness
                && self.round == other.round
                && self.signature == other.signature
        }
    }

    #[test]
    fn randomness_beacon_verification_success_works() {
        match chained_beacon().verify(chained_chain_info()) {
            Ok(ok) => assert!(ok),
            Err(_err) => panic!("Chained beacon should validate on chained info"),
        }

        match chained_beacon_1().verify(chained_chain_info()) {
            Ok(ok) => assert!(ok),
            Err(_err) => {
                panic!("Chained beacon should validate on chained info for the first beacon")
            }
        }

        match unchained_beacon().verify(unchained_chain_info()) {
            Ok(ok) => assert!(ok),
            Err(_err) => panic!("Unchained beacon should validate on unchained info"),
        }

        match unchained_beacon_on_g1().verify(unchained_chain_on_g1_info()) {
            Ok(ok) => assert!(ok),
            Err(_err) => panic!("Unchained beacon on G1 should validate on unchained info"),
        }

        match unchained_beacon_on_g1_rfc().verify(unchained_chain_on_g1_rfc_info()) {
            Ok(ok) => assert!(ok),
            Err(_err) => panic!("Unchained beacon on G1 RFC should validate on unchained info"),
        }
    }

    #[test]
    fn randomness_beacon_verification_failure_works() {
        match invalid_beacon().verify(chained_chain_info()) {
            Ok(ok) => assert!(!ok, "Invalid beacon should not validate"),
            Err(_err) => panic!("Invalid beacon should not validate without returning an error"),
        }

        match unchained_beacon().verify(chained_chain_info()) {
            Ok(ok) => assert!(!ok, "Unchained beacon should not validate on chained info"),
            Err(_err) => panic!(
                "Unchained beacon should not validate on chained info without returning an error"
            ),
        }

        match unchained_beacon_on_g1().verify(unchained_chain_info()) {
            Ok(ok) => assert!(!ok, "Unchained beacon on G1 should not validate on chained info"),
            Err(_err) => panic!(
                "Unchained beacon on G1 should not validate on chained info without returning an error"
            ),
        }

        // Regression test to confirm the introduction of RFC compliant chain does not break existing integration
        // Original change is on [drand/drand#1249](https://github.com/drand/drand/pull/1249)
        match unchained_beacon_on_g1().verify(unchained_chain_on_g1_rfc_info()) {
            Ok(ok) => assert!(!ok, "Unchained beacon on G1 (not RFC compliant) should not validate on unchained compliant info"),
            Err(_err) => panic!(
                "Unchained beacon on G1 (non Hash to curve RFC compliant) should not validate on unchained G1 info without returning an error"
            ),
        }

        match unchained_beacon_on_g1_rfc().verify(unchained_chain_on_g1_info()) {
            Ok(ok) => assert!(!ok, "Unchained beacon on G1 should not validate on unchained G1 (non Hash to curve RFC compliant) info"),
            Err(_err) => panic!(
                "Unchained beacon on G1 should not validate on unchained G1 (non Hash to curve RFC compliant) info without returning an error"
            ),
        }
    }

    #[test]
    fn randomness_beacon_time_success_works() {
        const FIRST_ROUND: u64 = 1;
        let chain = unchained_chain_info().into();
        let beacon_time = RandomnessBeaconTime::new(&chain, &FIRST_ROUND.to_string());
        assert!(
            beacon_time.round() == FIRST_ROUND,
            "Round number has been modified when computing its time"
        );
        assert!(
            beacon_time.absolute().unix_timestamp() as u64 == chain.genesis_time(),
            "Time of the first round must be genesis time"
        );
        assert!(
            beacon_time.relative().is_negative(),
            "First round should be before current time"
        );

        let genesis_beacon_time =
            RandomnessBeaconTime::new(&chain, &beacon_time.absolute().format(&Rfc3339).unwrap());
        assert!(
            genesis_beacon_time.round() == FIRST_ROUND,
            "Parsing genesis from absolute time should provide the first round"
        );
        assert!(
            genesis_beacon_time
                .relative()
                .abs()
                .sub(beacon_time.relative().abs())
                .is_positive(),
            "Parsing the same beacon at two different interval should advance relative time"
        );

        const FUTURE_ROUND: u64 = 10 * 1000 * 1000 * 1000; // attempt of max round. cannot use u64::MAX because we're going to perform multiplication and additions, which would go past the limit
        let chain = unchained_chain_info().into();
        let beacon_time = RandomnessBeaconTime::new(&chain, &FUTURE_ROUND.to_string());
        assert!(
            beacon_time.round() == FUTURE_ROUND,
            "Round number has been modified when computing its time"
        );
        assert!(
            beacon_time.absolute().unix_timestamp() as u64
                == chain.genesis_time() + (FUTURE_ROUND - 1) * chain.period(),
            "Time of a future round should be genesis + period"
        );
        assert!(
            beacon_time.relative().is_positive(),
            "Future round should be after current time"
        );

        const FUTURE_ROUND_RELATIVE: u64 = 10;
        const FUTURE_ROUND_RELATIVE_TIME: &str = "30s";
        let chain = unchained_chain_info().into();
        let beacon_time = RandomnessBeaconTime::new(&chain, "0s");
        let future_beacon_time = RandomnessBeaconTime::new(&chain, FUTURE_ROUND_RELATIVE_TIME);
        assert!(
            beacon_time.round() + FUTURE_ROUND_RELATIVE == future_beacon_time.round(),
            "Round number should match period*difference in round"
        );
        assert!(
            future_beacon_time
                .relative()
                .sub(beacon_time.relative())
                .whole_seconds()
                .to_string()
                + "s"
                == FUTURE_ROUND_RELATIVE_TIME,
            "Relative time parsing should be precise up to the second"
        );
    }
}
