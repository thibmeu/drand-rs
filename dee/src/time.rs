use anyhow::Result;
use colored::Colorize;
use drand_core::beacon::RandomnessBeaconTime;
use drand_core::{ChainOptions, HttpClient};
use time::format_description::well_known::Rfc3339;

use crate::config::ConfigChain;
use crate::print::Print;

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

pub fn round_from_option(
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
        None => client.latest()?.round().to_string(),
    };

    Ok(RandomnessBeaconTime::new(&info.into(), &round))
}
