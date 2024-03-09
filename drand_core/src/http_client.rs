use std::{str::FromStr, sync::Mutex};
use thiserror::Error;
#[cfg(feature = "time")]
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use url::Url;

#[cfg(feature = "time")]
use crate::beacon::RandomnessBeaconTime;
use crate::{
    beacon::{ApiBeacon, BeaconError, RandomnessBeacon},
    chain::{ChainInfo, ChainOptions},
    DrandError, Result,
};

#[derive(Error, Debug)]
pub enum HttpClientError {
    #[error("Chain info is invalid")]
    InvalidChainInfo,
    #[error("Failed to retrieve chain info {message}")]
    FailedToRetrieveChainInfo { message: String },
    #[error("{e}. You might need to add \"https://\" to the provided URL.")]
    NoProtocol { e: url::ParseError },
    #[error(transparent)]
    ParseURL(#[from] url::ParseError),
    #[error(transparent)]
    RequestFailed(#[from] Box<ureq::Error>),
}

/// HTTP Client for drand
/// Queries a specified HTTP endpoint given by `chain`, with specific `options`
/// By default, the client verifies answers, and caches retrieved chain informations
pub struct HttpClient {
    base_url: url::Url,
    options: ChainOptions,
    cached_chain_info: Mutex<Option<ChainInfo>>,
    http_client: ureq::Agent,
}

impl HttpClient {
    pub fn new(base_url: &str, options: Option<ChainOptions>) -> Result<Self> {
        // The most common error is when user forget to add protocol in front of the provided URL string.
        // The error provided by url::Url is rather obscure when that happens.
        let mut url = Url::parse(base_url).map_err(|e| {
            if e == url::ParseError::RelativeUrlWithoutBase {
                Box::new(HttpClientError::NoProtocol { e })
            } else {
                Box::new(HttpClientError::ParseURL(e))
            }
        })?;
        // Ensure base URL ends with a trailing slash.
        // Given it's the base for API calls, it allows for easier joins in other methods.
        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }
        Ok(Self {
            base_url: url,
            options: options.unwrap_or_default(),
            cached_chain_info: Mutex::new(None),
            http_client: ureq::AgentBuilder::new().build(),
        })
    }

    fn chain_info_no_cache(&self) -> Result<ChainInfo> {
        let response = self
            .http_client
            .get(
                self.base_url
                    .join("info")
                    .map_err(|e| -> DrandError { Box::new(HttpClientError::ParseURL(e)).into() })?
                    .as_str(),
            )
            .call()
            .map_err(|e| -> DrandError {
                Box::new(HttpClientError::RequestFailed(e.into())).into()
            })?;
        let info = if response.status() < 400 {
            response
                .into_json::<ChainInfo>()
                .map_err(|_| Box::new(BeaconError::Parsing))?
        } else {
            return Err(Box::new(HttpClientError::FailedToRetrieveChainInfo {
                message: response.into_string().unwrap_or_default(),
            })
            .into());
        };
        match self.options().verify(&info) {
            true => Ok(info),
            false => Err(Box::new(HttpClientError::InvalidChainInfo).into()),
        }
    }

    fn beacon_url(&self, round: String) -> Result<Url> {
        let mut url = self
            .base_url
            .join(&format!("public/{round}"))
            .map_err(|e| -> DrandError { Box::new(HttpClientError::ParseURL(e)).into() })?;
        if !self.options().is_cache() {
            url.query_pairs_mut()
                .append_key_only(format!("{}", rand::random::<u64>()).as_str());
        }
        Ok(url)
    }

    fn verify_beacon(&self, beacon: RandomnessBeacon, round: String) -> Result<RandomnessBeacon> {
        if !self.options().is_beacon_verification() {
            return Ok(beacon);
        }

        if !beacon.verify(self.chain_info()?)? {
            return Err(Box::new(BeaconError::Validation).into());
        }

        if round == "latest" {
            return Ok(beacon);
        }
        let round: u64 = round
            .parse()
            .map_err(|_| -> DrandError { Box::new(BeaconError::Parsing).into() })?;
        if beacon.round() != round {
            return Err(Box::new(BeaconError::RoundMismatch).into());
        }
        Ok(beacon)
    }

    fn get_with_string(&self, round: String) -> Result<RandomnessBeacon> {
        let beacon = self
            .http_client
            .get(self.beacon_url(round.clone())?.as_str())
            .call()
            .map_err(|e| -> DrandError {
                match e {
                    ureq::Error::Status(404, _) => Box::new(BeaconError::NotFound).into(),
                    _ => Box::new(HttpClientError::RequestFailed(e.into())).into(),
                }
            })?
            .into_json::<ApiBeacon>()
            .map_err(|_| -> DrandError { Box::new(BeaconError::Parsing).into() })?;

        let info = self.chain_info()?;
        let unix_time = info.genesis_time() + beacon.round() * info.period();
        let beacon = RandomnessBeacon::new(beacon, unix_time);

        self.verify_beacon(beacon, round)
    }

    pub fn base_url(&self) -> String {
        self.base_url.to_string()
    }

    pub fn options(&self) -> ChainOptions {
        self.options.clone()
    }

    pub fn chain_info(&self) -> Result<ChainInfo> {
        if self.options().is_cache() {
            let cached = self.cached_chain_info.lock().unwrap().to_owned();
            match cached {
                Some(info) => Ok(info),
                None => {
                    let info = self.chain_info_no_cache()?;
                    *self.cached_chain_info.lock().unwrap() = Some(info.clone());
                    Ok(info)
                }
            }
        } else {
            self.chain_info_no_cache()
        }
    }

    #[cfg(feature = "time")]
    pub fn latest(&self) -> Result<RandomnessBeacon> {
        // it is possible to either use round number 0, latest, or to infer the round number based on the current time
        // to allow for round verification, using inferance seems to be the best approach
        // without verification, latest is used instead
        if self.options().is_beacon_verification() {
            let info = self.chain_info()?;
            let now = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();
            let time =
                RandomnessBeaconTime::parse(&info.into(), &now).expect("time should be valid");
            self.get_with_string(time.round().to_string())
        } else {
            self.get_with_string("latest".to_owned())
        }
    }

    #[cfg(not(feature = "time"))]
    pub fn latest(&self) -> Result<RandomnessBeacon> {
        self.get_with_string("latest".to_owned())
    }

    pub fn get(&self, round_number: u64) -> Result<RandomnessBeacon> {
        self.get_with_string(round_number.to_string())
    }

    pub fn get_by_unix_time(&self, round_unix_time: u64) -> Result<RandomnessBeacon> {
        let info = self.chain_info()?;
        let round = (round_unix_time - info.genesis_time()) / info.period();

        self.get(round)
    }
}

impl crate::chain::ChainClient for HttpClient {
    fn options(&self) -> ChainOptions {
        self.options()
    }

    fn latest(&self) -> Result<RandomnessBeacon> {
        self.latest()
    }

    fn get(&self, round_number: u64) -> Result<RandomnessBeacon> {
        self.get(round_number)
    }

    fn chain_info(&self) -> Result<ChainInfo> {
        self.chain_info()
    }
}

impl TryFrom<&str> for HttpClient {
    type Error = DrandError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl FromStr for HttpClient {
    type Err = DrandError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::new(s, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon::{tests::chained_beacon, tests::invalid_beacon, tests::unchained_beacon};
    use crate::chain::{
        tests::chained_chain_info, tests::unchained_chain_info, ChainOptions, ChainVerification,
    };
    #[cfg(feature = "time")]
    use time::Duration;

    use super::*;

    #[test]
    fn client_no_cache_works() {
        let mut server = mockito::Server::new();
        let info_mock = server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_chain_info()).unwrap())
            .expect_at_least(2)
            .create();
        let expected_round = chained_beacon().round();
        let get_mock = server
            .mock("GET", format!("/public/{expected_round}").as_str())
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_beacon()).unwrap())
            .expect_at_least(2)
            .create();

        // test client without cache
        let no_cache_client = HttpClient::new(
            server.url().as_str(),
            Some(ChainOptions::new(true, false, None)),
        )
        .unwrap();

        // info endpoint
        let info = match no_cache_client.chain_info() {
            Ok(info) => info,
            Err(_err) => panic!("fetch should have succeded"),
        };
        assert_eq!(info, chained_chain_info());
        // do it again to see if it's cached or not
        let _ = no_cache_client.chain_info();
        info_mock.assert();

        // get endpoint
        let beacon = match no_cache_client.get(expected_round) {
            Ok(beacon) => beacon,
            Err(_err) => panic!("fetch should have succeded"),
        };
        assert_eq!(beacon.beacon(), chained_beacon());
        assert_eq!(beacon.time(), 1625431050);
        // do it again to see if it's cached or not
        let _ = no_cache_client.get(expected_round);
        get_mock.assert();
    }

    #[test]
    fn client_cache_works() {
        let mut server = mockito::Server::new();
        let info_mock = server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_chain_info()).unwrap())
            .expect_at_least(1)
            .create();
        let expected_round = chained_beacon().round();
        let get_mock = server
            .mock("GET", format!("/public/{expected_round}").as_str())
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_beacon()).unwrap())
            .expect_at_least(1)
            .create();

        // test client with cache
        let cache_client = HttpClient::new(
            server.url().as_str(),
            Some(ChainOptions::new(false, true, None)),
        )
        .unwrap();

        // info endpoint
        let info = match cache_client.chain_info() {
            Ok(info) => info,
            Err(_err) => panic!("fetch should have succeded"),
        };
        assert_eq!(info, chained_chain_info());
        // do it again to see if it's cached or not
        let _ = cache_client.chain_info();
        info_mock.assert();

        // get endpoint
        let beacon = match cache_client.get(expected_round) {
            Ok(beacon) => beacon,
            Err(_err) => panic!("fetch should have succeded"),
        };
        assert_eq!(beacon.beacon(), chained_beacon());
        assert_eq!(beacon.time(), 1625431050);
        // do it again to see if it's cached or not
        let _ = cache_client.get(expected_round);
        get_mock.assert();
    }

    // Fakes the chain info genesis time so that the provided beacon is the latest one for the current time
    pub fn chain_info_with_latest(beacon: &ApiBeacon) -> ChainInfo {
        let info = unchained_chain_info();
        let latest_round = beacon.round();
        let period = info.period();
        let genesis_time = OffsetDateTime::now_utc()
            .checked_sub(Duration::seconds((latest_round * period - 1) as i64))
            .unwrap()
            .unix_timestamp();
        serde_json::from_str(&format!(
            r#"{{
            "public_key": "{public_key}",
            "period": {period},
            "genesis_time": {genesis_time},
            "hash": "{hash}",
            "groupHash": "{group_hash}",
            "schemeID": "{scheme_id}",
            "metadata": {{
                "beaconID": "default"
            }}
        }}"#,
            public_key = hex::encode(info.public_key()),
            hash = hex::encode(info.hash()),
            group_hash = hex::encode(info.group_hash()),
            scheme_id = info.scheme_id()
        ))
        .unwrap()
    }

    #[test]
    fn client_beacon_verification_works() {
        // unchained beacon
        let mut valid_server = mockito::Server::new();
        let _info_mock = valid_server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chain_info_with_latest(&unchained_beacon())).unwrap())
            .expect_at_least(1)
            .create();
        let _latest_mock = valid_server
            .mock("GET", "/public/latest")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_beacon()).unwrap())
            .expect_at_least(1)
            .create();
        let expected_round = unchained_beacon().round();
        let _get_mock = valid_server
            .mock("GET", format!("/public/{expected_round}").as_str())
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_beacon()).unwrap())
            .expect_at_least(1)
            .create();

        // test client without cache
        let client = HttpClient::new(
            valid_server.url().as_str(),
            Some(ChainOptions::new(true, false, None)),
        )
        .unwrap();

        // latest endpoint
        let latest = match client.latest() {
            Ok(beacon) => beacon,
            Err(err) => panic!("fetch should have succeded {}", err),
        };
        assert_eq!(latest.beacon(), unchained_beacon());

        // test client with round verification
        let client = HttpClient::new(
            valid_server.url().as_str(),
            Some(ChainOptions::new(true, false, None)),
        )
        .unwrap();

        match client.get(expected_round) {
            Ok(beacon) => assert_eq!(beacon.beacon(), unchained_beacon()),
            Err(err) => panic!("fetch should have succeded {}", err),
        }

        let mut invalid_server = mockito::Server::new();
        let _info_mock = invalid_server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_chain_info()).unwrap())
            .expect_at_least(1)
            .create();
        let expected_round = invalid_beacon().round();
        let _get_mock = invalid_server
            .mock("GET", format!("/public/{expected_round}").as_str())
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&invalid_beacon()).unwrap())
            .expect_at_least(1)
            .create();

        // test client without cache
        let client = HttpClient::new(
            invalid_server.url().as_str(),
            Some(ChainOptions::new(true, false, None)),
        )
        .unwrap();

        // get endpoint
        match client.get(expected_round) {
            Ok(_beacon) => panic!("Beacon should not validate"),
            Err(_err) => (),
        }
    }

    #[test]
    fn client_chain_verification_works() {
        // unchained beacon
        let mut valid_server = mockito::Server::new();
        let _info_mock = valid_server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_chain_info()).unwrap())
            .expect_at_least(1)
            .create();
        let _latest_mock = valid_server
            .mock("GET", "/public/latest")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_beacon()).unwrap())
            .expect_at_least(1)
            .create();
        let expected_round = chained_beacon().round();
        let _get_mock = valid_server
            .mock("GET", format!("/public/{expected_round}").as_str())
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_beacon()).unwrap())
            .expect_at_least(1)
            .create();
        let invalid_round = expected_round + 1;
        let _get_mock = valid_server
            .mock("GET", format!("/public/{invalid_round}").as_str())
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_beacon()).unwrap())
            .expect_at_least(1)
            .create();

        // test client without cache
        let unchained_info = unchained_chain_info();
        let unchained_client = HttpClient::new(
            valid_server.url().as_str(),
            Some(ChainOptions::new(
                true,
                false,
                Some(ChainVerification::new(
                    Some(unchained_info.hash()),
                    Some(unchained_info.public_key()),
                )),
            )),
        )
        .unwrap();

        // get endpoint
        let beacon = match unchained_client.get(expected_round) {
            Ok(beacon) => beacon,
            Err(err) => panic!("fetch should have succeded {}", err),
        };
        assert_eq!(beacon.beacon(), unchained_beacon());
        assert_eq!(beacon.time(), 1654677099);

        // test with not the correct hash
        let chained_info = chained_chain_info();
        let invalid_client = HttpClient::new(
            valid_server.url().as_str(),
            Some(ChainOptions::new(
                true,
                false,
                Some(ChainVerification::new(Some(chained_info.hash()), None)),
            )),
        )
        .unwrap();

        match invalid_client.get(expected_round) {
            Ok(_beacon) => panic!("Beacon should not validate"),
            Err(_err) => (),
        };
        // test with not the correct public_key
        let chained_info = chained_chain_info();
        let invalid_client = HttpClient::new(
            valid_server.url().as_str(),
            Some(ChainOptions::new(
                true,
                false,
                Some(ChainVerification::new(
                    None,
                    Some(chained_info.public_key()),
                )),
            )),
        )
        .unwrap();

        match invalid_client.get(expected_round) {
            Ok(_beacon) => panic!("Beacon should not validate"),
            Err(_err) => (),
        };

        // test with incorrect round
        let chained_info = chained_chain_info();
        let invalid_client = HttpClient::new(
            valid_server.url().as_str(),
            Some(ChainOptions::new(
                true,
                false,
                Some(ChainVerification::new(
                    Some(chained_info.hash()),
                    Some(chained_info.public_key()),
                )),
            )),
        )
        .unwrap();

        match invalid_client.get(invalid_round) {
            Ok(_beacon) => panic!("Beacon should not validate"),
            Err(_err) => (),
        };
    }
}
