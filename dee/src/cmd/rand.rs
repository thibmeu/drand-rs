use std::cmp::Ordering;

use anyhow::Result;

use colored::Colorize;
use drand_core::{
    beacon::RandomnessBeacon,
    chain::{self, ChainOptions},
    http_chain_client::HttpChainClient,
};
use serde::{Deserialize, Serialize};

use crate::{
    config::{self, ConfigChain},
    print::{print_with_format, Format, Print},
    time::RandomnessBeaconTime,
};

#[derive(Serialize, Deserialize)]
struct RandResult {
    beacon: RandomnessBeacon,
    time: RandomnessBeaconTime,
}

impl Print for RandResult {
    fn short(&self) -> Result<String> {
        Ok(hex::encode(self.beacon.randomness()))
    }
    fn long(&self) -> Result<String> {
        let relative = self.time.relative();
        let seconds = relative.num_seconds().abs() % 60;
        let minutes = (relative.num_minutes()).abs() % 60;
        let hours = relative.num_hours().abs();
        let epoch = match relative.num_seconds().cmp(&0) {
            Ordering::Less => "ago",
            Ordering::Equal => "now",
            Ordering::Greater => "from now",
        };
        let relative = format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2} {epoch}");
        Ok(format!(
            r"{: <10}: {}
{: <10}: {}
{: <10}: {}
{: <10}: {}
{: <10}: {}",
            "Round".bold(),
            self.time.round(),
            "Relative".bold(),
            relative,
            "Absolute".bold(),
            self.time.absolute(),
            "Randomness".bold(),
            hex::encode(self.beacon.randomness()),
            "Signature".bold(),
            hex::encode(self.beacon.signature()),
        ))
    }

    fn json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self.beacon)?)
    }
}

pub async fn rand(
    _cfg: &config::Local,
    format: Format,
    chain: ConfigChain,
    beacon: Option<u64>,
    verify: bool,
) -> Result<String> {
    let chain = chain::Chain::new(&chain.url());
    let info = chain.info().await?;

    let client = HttpChainClient::new(
        chain,
        Some(ChainOptions::new(verify, true, Some(info.clone().into()))),
    );

    let beacon = match beacon {
        Some(round) => client.get(round).await?,
        None => client.latest().await?,
    };

    let time = RandomnessBeaconTime::from_round(&info, beacon.round());

    print_with_format(RandResult { beacon, time }, format)
}
