use anyhow::{anyhow, Result};
use std::sync::Mutex;

use crate::{
    beacon::RandomnessBeacon,
    chain::{Chain, ChainInfo, ChainOptions},
};

/// HTTP Client for drand
/// Queries a specified HTTP endpoint given by `chain`, with specific `options`
/// By default, the client verifies answers, and caches retrieved chain informations
pub struct HttpChainClient {
    chain: Chain,
    options: ChainOptions,
    cached_chain_info: Mutex<Option<ChainInfo>>,
    http_client: reqwest::Client,
}

impl HttpChainClient {
    pub fn new(chain: Chain, options: Option<ChainOptions>) -> Self {
        Self {
            chain,
            options: options.unwrap_or_default(),
            cached_chain_info: Mutex::new(None),
            http_client: reqwest::Client::builder().build().unwrap(),
        }
    }

    async fn chain_info_no_cache(&self) -> Result<ChainInfo> {
        let info = self.chain.info().await?;
        match self.options().verify(info.clone()) {
            true => Ok(info),
            false => Err(anyhow!("Chain info is invalid")),
        }
    }

    async fn chain_info(&self) -> Result<ChainInfo> {
        if self.options().is_cache() {
            let cached = self.cached_chain_info.lock().unwrap().to_owned();
            match cached {
                Some(info) => Ok(info),
                None => {
                    let info = self.chain_info_no_cache().await?;
                    *self.cached_chain_info.lock().unwrap() = Some(info.clone());
                    Ok(info)
                }
            }
        } else {
            self.chain_info_no_cache().await
        }
    }

    fn beacon_url(&self, round: String) -> Result<reqwest::Url> {
        let url: reqwest::Url = self.chain.base_url().as_str().try_into()?;
        let mut url = url.join(&format!("public/{round}"))?;
        if !self.options().is_cache() {
            url.query_pairs_mut()
                .append_key_only(format!("{}", rand::random::<u64>()).as_str());
        }
        Ok(url)
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

    pub fn options(&self) -> ChainOptions {
        self.options.clone()
    }

    pub async fn latest(&self) -> Result<RandomnessBeacon> {
        let beacon = self
            .http_client
            .get(self.beacon_url("latest".to_string())?)
            .send()
            .await?
            .json::<RandomnessBeacon>()
            .await?;

        self.verify_beacon(beacon).await
    }

    pub async fn get(&self, round_number: u64) -> Result<RandomnessBeacon> {
        let beacon = self
            .http_client
            .get(self.beacon_url(round_number.to_string())?)
            .send()
            .await?
            .json::<RandomnessBeacon>()
            .await?;

        self.verify_beacon(beacon).await
    }

    pub fn chain(&self) -> Chain {
        self.chain.clone()
    }
}

impl From<Chain> for HttpChainClient {
    fn from(value: Chain) -> Self {
        Self::new(value, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon::{tests::chained_beacon, tests::invalid_beacon, tests::unchained_beacon};
    use crate::chain::{
        tests::chained_chain_info, tests::unchained_chain_info, Chain, ChainOptions,
        ChainVerification,
    };

    use super::*;

    #[tokio::test]
    async fn client_no_cache_works() {
        let mut server = mockito::Server::new_async().await;
        let info_mock = server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_chain_info()).unwrap())
            .expect_at_least(2)
            .create_async()
            .await;
        let latest_mock = server
            .mock("GET", "/public/latest")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_beacon()).unwrap())
            .expect_at_least(2)
            .create_async()
            .await;

        let chain: Chain = server.url().as_str().try_into().unwrap();

        // test client without cache
        let no_cache_client =
            HttpChainClient::new(chain.clone(), Some(ChainOptions::new(true, false, None)));

        // info endpoint
        let info = match no_cache_client.chain_info().await {
            Ok(info) => info,
            Err(_err) => panic!("fetch should have succeded"),
        };
        assert_eq!(info, chained_chain_info());
        // do it again to see if it's cached or not
        let _ = no_cache_client.chain_info().await;
        info_mock.assert_async().await;

        // latest endpoint
        let latest = match no_cache_client.latest().await {
            Ok(beacon) => beacon,
            Err(_err) => panic!("fetch should have succeded"),
        };
        assert_eq!(latest, chained_beacon());
        // do it again to see if it's cached or not
        let _ = no_cache_client.latest().await;
        latest_mock.assert_async().await;
    }

    #[tokio::test]
    async fn client_cache_works() {
        let mut server = mockito::Server::new_async().await;
        let info_mock = server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_chain_info()).unwrap())
            .expect_at_least(1)
            .create_async()
            .await;
        let latest_mock = server
            .mock("GET", "/public/latest")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_beacon()).unwrap())
            .expect_at_least(1)
            .create_async()
            .await;

        let chain: Chain = server.url().as_str().try_into().unwrap();

        // test client with cache
        let cache_client =
            HttpChainClient::new(chain.clone(), Some(ChainOptions::new(true, true, None)));

        // info endpoint
        let info = match cache_client.chain_info().await {
            Ok(info) => info,
            Err(_err) => panic!("fetch should have succeded"),
        };
        assert_eq!(info, chained_chain_info());
        // do it again to see if it's cached or not
        let _ = cache_client.chain_info().await;
        info_mock.assert_async().await;

        // latest endpoint
        let latest = match cache_client.latest().await {
            Ok(beacon) => beacon,
            Err(_err) => panic!("fetch should have succeded"),
        };
        assert_eq!(latest, chained_beacon());
        // do it again to see if it's cached or not
        let _ = cache_client.latest().await;
        latest_mock.assert_async().await;
    }

    #[tokio::test]
    async fn client_beacon_verification_works() {
        // unchained beacon
        let mut valid_server = mockito::Server::new_async().await;
        let _info_mock = valid_server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_chain_info()).unwrap())
            .expect_at_least(1)
            .create_async()
            .await;
        let _latest_mock = valid_server
            .mock("GET", "/public/latest")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_beacon()).unwrap())
            .expect_at_least(1)
            .create_async()
            .await;

        let valid_chain: Chain = valid_server.url().as_str().try_into().unwrap();

        // test client without cache
        let client = HttpChainClient::new(
            valid_chain.clone(),
            Some(ChainOptions::new(true, false, None)),
        );

        // latest endpoint
        let latest = match client.latest().await {
            Ok(beacon) => beacon,
            Err(err) => panic!("fetch should have succeded {}", err),
        };
        assert_eq!(latest, unchained_beacon());

        let mut invalid_server = mockito::Server::new_async().await;
        let _info_mock = invalid_server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&chained_chain_info()).unwrap())
            .expect_at_least(1)
            .create_async()
            .await;
        let _latest_mock = invalid_server
            .mock("GET", "/public/latest")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&invalid_beacon()).unwrap())
            .expect_at_least(1)
            .create_async()
            .await;

        let invalid_chain: Chain = invalid_server.url().as_str().try_into().unwrap();

        // test client without cache
        let client = HttpChainClient::new(
            invalid_chain.clone(),
            Some(ChainOptions::new(true, false, None)),
        );

        // latest endpoint
        match client.latest().await {
            Ok(_beacon) => panic!("Beacon should not validate"),
            Err(_err) => (),
        }
    }

    #[tokio::test]
    async fn client_chain_verification_works() {
        // unchained beacon
        let mut valid_server = mockito::Server::new_async().await;
        let _info_mock = valid_server
            .mock("GET", "/info")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_chain_info()).unwrap())
            .expect_at_least(1)
            .create_async()
            .await;
        let _latest_mock = valid_server
            .mock("GET", "/public/latest")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&unchained_beacon()).unwrap())
            .expect_at_least(1)
            .create_async()
            .await;

        let unchained_chain: Chain = valid_server.url().as_str().try_into().unwrap();

        // test client without cache
        let unchained_info = unchained_chain_info();
        let unchained_client = HttpChainClient::new(
            unchained_chain.clone(),
            Some(ChainOptions::new(
                true,
                false,
                Some(ChainVerification::new(
                    Some(unchained_info.hash()),
                    Some(unchained_info.public_key()),
                )),
            )),
        );

        // latest endpoint
        let latest = match unchained_client.latest().await {
            Ok(beacon) => beacon,
            Err(err) => panic!("fetch should have succeded {}", err),
        };
        assert_eq!(latest, unchained_beacon());

        // test with not the correct hash
        let chained_info = chained_chain_info();
        let invalid_client = HttpChainClient::new(
            unchained_chain.clone(),
            Some(ChainOptions::new(
                true,
                false,
                Some(ChainVerification::new(Some(chained_info.hash()), None)),
            )),
        );

        match invalid_client.latest().await {
            Ok(_beacon) => panic!("Beacon should not validate"),
            Err(_err) => (),
        };
        // test with not the correct public_key
        let chained_info = chained_chain_info();
        let invalid_client = HttpChainClient::new(
            unchained_chain.clone(),
            Some(ChainOptions::new(
                true,
                false,
                Some(ChainVerification::new(
                    None,
                    Some(chained_info.public_key()),
                )),
            )),
        );

        match invalid_client.latest().await {
            Ok(_beacon) => panic!("Beacon should not validate"),
            Err(_err) => (),
        };
    }
}
