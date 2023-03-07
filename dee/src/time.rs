use anyhow::{anyhow, Result};
use chrono::FixedOffset;
use colored::Colorize;
use drand_core::chain::{self, ChainClient, ChainInfo, ChainOptions};
use drand_core::http_chain_client::HttpChainClient;
use serde::{Deserialize, Serialize};

use crate::config::ConfigChain;
use crate::print::Print;

fn parse_duration(duration: &str) -> Result<chrono::Duration> {
    let l = duration.len() - 1;
    let principal = duration[0..l].parse::<i64>()?;

    let duration = match duration.chars().last().unwrap() {
        's' => chrono::Duration::seconds(principal),
        'm' => chrono::Duration::minutes(principal),
        'h' => chrono::Duration::hours(principal),
        'd' => chrono::Duration::days(principal),
        _ => return Err(anyhow!("cannot parse duration")),
    };
    Ok(duration)
}

mod date_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        chrono::NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

mod duration_format {
    use chrono::Duration;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", duration.num_seconds());
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        super::parse_duration(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RandomnessBeaconTime {
    round: u64,
    #[serde(with = "duration_format")]
    relative: chrono::Duration,
    #[serde(with = "date_format")]
    absolute: chrono::NaiveDateTime,
}

impl RandomnessBeaconTime {
    pub fn new(info: &ChainInfo, round: &str) -> Self {
        match (
            round.parse::<u64>(),
            parse_duration(round),
            chrono::DateTime::parse_from_rfc3339(round),
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

    pub fn relative(&self) -> chrono::Duration {
        self.relative
    }

    pub fn absolute(&self) -> chrono::NaiveDateTime {
        self.absolute
    }

    pub fn from_round(info: &ChainInfo, round: u64) -> Self {
        let genesis =
            chrono::NaiveDateTime::from_timestamp_opt(info.genesis_time() as i64, 0).unwrap();
        let absolute = genesis + chrono::Duration::seconds((round * info.period()) as i64);
        let relative = absolute - chrono::Utc::now().naive_utc();
        Self {
            round,
            relative,
            absolute,
        }
    }

    fn from_duration(info: &ChainInfo, relative: chrono::Duration) -> Self {
        let genesis =
            chrono::NaiveDateTime::from_timestamp_opt(info.genesis_time() as i64, 0).unwrap();

        let absolute = chrono::Utc::now().naive_utc() + relative;
        let round = ((absolute - genesis).num_seconds() / (info.period() as i64)) as u64;

        Self {
            round,
            relative,
            absolute,
        }
    }

    fn from_datetime(info: &ChainInfo, absolute: chrono::DateTime<FixedOffset>) -> Self {
        let absolute = chrono::NaiveDateTime::from_timestamp_opt(
            absolute.timestamp(),
            absolute.timestamp_subsec_nanos(),
        )
        .unwrap();
        let genesis =
            chrono::NaiveDateTime::from_timestamp_opt(info.genesis_time() as i64, 0).unwrap();

        let relative = absolute - chrono::Utc::now().naive_utc();
        let round = ((absolute - genesis).num_seconds() / (info.period() as i64)) as u64;

        Self {
            round,
            relative,
            absolute,
        }
    }
}

impl Print for RandomnessBeaconTime {
    fn pretty(&self) -> Result<String> {
        Ok(format!(
            r"{: <10}: {}
{: <10}: {}
{: <10}: {}",
            "Round".bold(),
            self.round(),
            "Relative".bold(),
            self.relative(),
            "Absolute".bold(),
            self.absolute().format("%Y-%m-%d %H:%M:%S"),
        ))
    }

    fn json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

pub async fn round_from_option(
    chain: ConfigChain,
    round: Option<String>,
) -> Result<RandomnessBeaconTime> {
    let info = chain.info();
    let chain = chain::Chain::new(&chain.url());

    let client = HttpChainClient::new(
        chain,
        Some(ChainOptions::new(true, true, Some(info.clone().into()))),
    );

    let round = match round {
        Some(round) => round,
        None => client.latest().await?.round().to_string(),
    };

    Ok(RandomnessBeaconTime::new(&info, &round))
}
