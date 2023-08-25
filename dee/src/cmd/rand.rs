use std::cmp::Ordering;

use anyhow::{anyhow, Result};

use colored::Colorize;
use drand_core::{
    beacon::{BeaconError, RandomnessBeacon, RandomnessBeaconTime},
    ChainOptions, DrandError, HttpClient,
};
use serde::Serialize;

use crate::{
    config::{self, ConfigChain},
    print::{print_with_format, Format, Print},
};

#[derive(Serialize)]
pub(crate) struct RandResult {
    beacon: Option<RandomnessBeacon>,
    time: RandomnessBeaconTime,
}

impl RandResult {
    pub(crate) fn new(beacon: Option<RandomnessBeacon>, time: RandomnessBeaconTime) -> Self {
        Self { beacon, time }
    }
}

impl Print for RandResult {
    fn short(&self) -> Result<String> {
        match self.beacon.as_ref() {
            Some(beacon) => Ok(hex::encode(beacon.randomness())),
            None => {
                let relative = self.time.relative();
                let seconds = relative.whole_seconds().abs() % 60;
                let minutes = relative.whole_minutes().abs() % 60;
                let hours = relative.whole_hours().abs();
                let relative = format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2}");
                Err(anyhow!(
                    "Too early. Beacon round is {}, estimated in {} ({}).",
                    self.time.round(),
                    relative,
                    self.time.absolute(),
                ))
            }
        }
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
        let mut output = format!(
            r"{: <10}: {}
{: <10}: {}
{: <10}: {}",
            "Round".bold(),
            self.time.round(),
            "Relative".bold(),
            relative,
            "Absolute".bold(),
            self.time.absolute(),
        );
        if let Some(beacon) = self.beacon.as_ref() {
            output = format!(
                r"{output}
{: <10}: {}
{: <10}: {}",
                "Randomness".bold(),
                hex::encode(beacon.randomness()),
                "Signature".bold(),
                hex::encode(beacon.signature()),
            );
        }
        Ok(output)
    }

    fn json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self.beacon)?)
    }
}

pub fn rand(
    _cfg: &config::Local,
    format: Format,
    chain: ConfigChain,
    beacon: Option<String>,
    verify: bool,
) -> Result<String> {
    let base_url = chain.url();
    let info = chain.info();
    let latest = beacon.is_none();

    let beacon = beacon.unwrap_or("0s".to_owned());
    let time = RandomnessBeaconTime::new(&info.clone().into(), &beacon);

    let client = HttpClient::new(
        &base_url,
        Some(ChainOptions::new(verify, true, Some(info.into()))),
    )?;

    let beacon = if latest {
        client.latest()
    } else {
        client.get(time.round())
    };

    match beacon {
        Ok(beacon) => print_with_format(RandResult::new(Some(beacon), time), format),
        Err(DrandError::Beacon(e)) => match *e {
            BeaconError::NotFound => print_with_format(RandResult::new(None, time), format),
            _ => Ok(e.to_string()),
        },
        Err(e) => Err(e.into()),
    }
}
