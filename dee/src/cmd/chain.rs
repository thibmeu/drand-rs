use anyhow::{anyhow, Result};
use chrono::{TimeZone, Utc};
use colored::Colorize;
use drand_core::chain::{Chain, ChainInfo};
use log::{log_enabled, Level};

use crate::{
    config::{self, ConfigChain},
    print::{self, print_with_format},
};

pub async fn add(cfg: &mut config::Local, name: String, url: String) -> Result<String> {
    let chain = Chain::new(&url);
    let info = chain.info().await?;

    cfg.add_chain(name.clone(), ConfigChain::new(url, info))?;

    Ok(name)
}

pub fn remove(cfg: &mut config::Local, name: String) -> Result<String> {
    cfg.remove_chain(name.clone())?;

    Ok(name)
}

pub fn rename(cfg: &mut config::Local, old: String, new: String) -> Result<String> {
    cfg.rename_chain(old, new.clone())?;

    Ok(new)
}

pub fn set_url(cfg: &mut config::Local, name: String, url: String) -> Result<String> {
    cfg.set_url_chain(name.clone(), url)?;

    Ok(name)
}

impl print::Print for ChainInfo {
    fn pretty(&self) -> Result<String> {
        Ok(format!(
            r#"{}: {}
{}: {}s
{}: {}
{}: {}
{}: {}
{}: {}
{}: {}"#,
            "Public Key".bold(),
            self.public_key(),
            "Period".bold(),
            self.period(),
            "Genesis".bold(),
            Utc.timestamp_opt(self.genesis_time() as i64, 0).unwrap(),
            "Chain Hash".bold(),
            self.hash(),
            "Group Hash".bold(),
            self.group_hash(),
            "Scheme ID".bold(),
            self.scheme_id(),
            "Beacon ID".bold(),
            self.metadata().beacon_id()
        ))
    }

    fn json(&self) -> Result<String> {
        serde_json::to_string(&self).map_err(|e| anyhow!(e))
    }
}

pub fn info(cfg: &config::Local, format: print::Format, name: String) -> Result<String> {
    let chain = match cfg.chain(&name) {
        Some(chain) => chain,
        None => return Err(anyhow!("Chain does not exist")),
    };

    print_with_format(chain.info(), format)
}

pub fn list(cfg: &config::Local) -> Result<String> {
    let chains: Vec<String> = cfg.chains().keys().cloned().collect();
    if chains.is_empty() {
        Ok("No chain".to_string())
    } else {
        let output: Vec<String> = chains
            .iter()
            .map(|chain| (chain.to_owned(), cfg.chain(chain.as_str()).unwrap().url()))
            .map(|(chain, url)| {
                if log_enabled!(Level::Warn) {
                    format!("{chain}\t{url}")
                } else {
                    chain
                }
            })
            .collect();
        Ok(output.join("\n"))
    }
}
