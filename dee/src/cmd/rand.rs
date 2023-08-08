use std::cmp::Ordering;

use anyhow::Result;

use colored::Colorize;
use drand_core::{
    beacon::{RandomnessBeacon, RandomnessBeaconTime},
    ChainOptions, HttpClient,
};
use serde::Serialize;

use crate::{
    config::{self, ConfigChain},
    print::{print_with_format, Format, Print},
};

#[derive(Serialize)]
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
        let seconds = relative.whole_seconds().abs() % 60;
        let minutes = (relative.whole_minutes()).abs() % 60;
        let hours = relative.whole_hours().abs();
        let epoch = match relative.whole_seconds().cmp(&0) {
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

pub fn rand(
    _cfg: &config::Local,
    format: Format,
    chain: ConfigChain,
    beacon: Option<u64>,
    verify: bool,
) -> Result<String> {
    let base_url = chain.url();
    let info = chain.info();

    let client = HttpClient::new(
        &base_url,
        Some(ChainOptions::new(verify, true, Some(info.clone().into()))),
    )?;

    let beacon = match beacon {
        Some(round) => client.get(round)?,
        None => client.latest()?,
    };

    let time = RandomnessBeaconTime::from_round(&info.into(), beacon.round());

    print_with_format(RandResult { beacon, time }, format)
}
