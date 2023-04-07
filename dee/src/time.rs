use anyhow::{anyhow, Result};
use colored::Colorize;
use drand_core::chain::ChainInfo;
use drand_core::{ChainOptions, HttpClient};
use serde::{Deserialize, Serialize};
use time::ext::NumericalDuration;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use crate::config::ConfigChain;
use crate::print::Print;

fn parse_duration(duration: &str) -> Result<Duration> {
    let l = duration.len() - 1;
    let principal = duration[0..l].parse::<i64>()?;

    let duration = match duration.chars().last().unwrap() {
        's' => principal.seconds(),
        'm' => principal.minutes(),
        'h' => principal.hours(),
        'd' => principal.days(),
        _ => return Err(anyhow!("cannot parse duration")),
    };
    Ok(duration)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RandomnessBeaconTime {
    round: u64,
    relative: Duration,
    #[serde(with = "time::serde::rfc3339")]
    absolute: OffsetDateTime,
}

impl RandomnessBeaconTime {
    pub fn new(info: &ChainInfo, round: &str) -> Self {
        match (
            round.parse::<u64>(),
            parse_duration(round),
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

    pub fn from_round(info: &ChainInfo, round: u64) -> Self {
        let genesis = OffsetDateTime::from_unix_timestamp(info.genesis_time() as i64).unwrap();

        let absolute = genesis + (((round - 1) * info.period()) as i64).seconds();
        let relative = absolute - OffsetDateTime::now_utc();
        Self {
            round,
            relative,
            absolute,
        }
    }

    fn from_duration(info: &ChainInfo, relative: Duration) -> Self {
        let genesis = OffsetDateTime::from_unix_timestamp(info.genesis_time() as i64).unwrap();

        let absolute = OffsetDateTime::now_utc() + relative;
        let round = ((absolute - genesis).whole_seconds() / (info.period() as i64) + 1) as u64;

        Self {
            round,
            relative,
            absolute,
        }
    }

    fn from_datetime(info: &ChainInfo, absolute: OffsetDateTime) -> Self {
        let genesis = OffsetDateTime::from_unix_timestamp(info.genesis_time() as i64).unwrap();

        let relative = absolute - OffsetDateTime::now_utc();
        let round = ((absolute - genesis).whole_seconds() / (info.period() as i64) + 1) as u64;

        Self {
            round,
            relative,
            absolute,
        }
    }
}

impl Print for RandomnessBeaconTime {
    fn short(&self) -> Result<String> {
        Ok(format!("{}", self.round()))
    }

    fn long(&self) -> Result<String> {
        Ok(format!(
            r"{: <10}: {}
{: <10}: {}
{: <10}: {}",
            "Round".bold(),
            self.round(),
            "Relative".bold(),
            self.relative(),
            "Absolute".bold(),
            self.absolute().format(&Rfc3339)?,
        ))
    }

    fn json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

pub async fn round_from_option(
    chain: &ConfigChain,
    round: Option<String>,
) -> Result<RandomnessBeaconTime> {
    let info = chain.info();

    let client = HttpClient::new(
        &chain.url(),
        Some(ChainOptions::new(true, true, Some(info.clone().into()))),
    )?;

    let round = match round {
        Some(round) => round,
        None => client.latest().await?.round().to_string(),
    };

    Ok(RandomnessBeaconTime::new(&info, &round))
}
