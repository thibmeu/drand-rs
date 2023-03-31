use anyhow::{anyhow, Result};
use chrono::{TimeZone, Utc};
use colored::Colorize;
use drand_core::HttpClient;
use log::{log_enabled, Level};

use crate::{
    config::{self, ConfigChain},
    print::{self, print_with_format},
};

pub async fn add(cfg: &mut config::Local, name: String, url: &str) -> Result<String> {
    if cfg.chain(&name).is_some() {
        return Err(anyhow!("remote {name} already exists."));
    }
    let client: HttpClient = url.try_into()?;
    let info = client.chain_info().await.map_err(|err| {
        anyhow!("failed to retrieve information from remote '{name}'. server response: {err}")
    })?;

    cfg.add_chain(name.clone(), ConfigChain::new(url, info))?;

    Ok(name)
}

pub fn remove(cfg: &mut config::Local, name: String) -> Result<String> {
    if cfg.chain(&name).is_none() {
        return Err(anyhow!("no such remote '{name}'."));
    }
    cfg.remove_chain(name.clone())?;

    Ok(name)
}

pub fn rename(cfg: &mut config::Local, old: String, new: String) -> Result<String> {
    if cfg.chain(&old).is_none() {
        return Err(anyhow!("no such remote '{old}'."));
    }
    if cfg.chain(&new).is_some() {
        return Err(anyhow!("remote {new} already exists."));
    }

    cfg.rename_chain(old.clone(), new.clone())?;

    if let Some(upstream) = cfg.upstream() {
        if upstream == old {
            cfg.set_upstream(&new)?;
        }
    }

    Ok(new)
}

pub fn set_url(cfg: &mut config::Local, name: String, url: &str) -> Result<String> {
    if cfg.chain(&name).is_none() {
        return Err(anyhow!("no such remote '{name}'."));
    }
    cfg.set_url_chain(name.clone(), url)?;

    Ok(name)
}

impl print::Print for ConfigChain {
    fn short(&self) -> Result<String> {
        Ok(hex::encode(self.info().public_key()))
    }

    fn long(&self) -> Result<String> {
        let info = self.info();
        Ok(format!(
            r"{: <10}: {}
{: <10}: {}
{: <10}: {}s
{: <10}: {}
{: <10}: {}
{: <10}: {}
{: <10}: {}
{: <10}: {}",
            "URL".bold(),
            self.url(),
            "Public Key".bold(),
            hex::encode(info.public_key()),
            "Period".bold(),
            info.period(),
            "Genesis".bold(),
            Utc.timestamp_opt(info.genesis_time() as i64, 0).unwrap(),
            "Chain Hash".bold(),
            hex::encode(info.hash()),
            "Group Hash".bold(),
            hex::encode(info.group_hash()),
            "Scheme ID".bold(),
            info.scheme_id(),
            "Beacon ID".bold(),
            info.metadata().beacon_id()
        ))
    }

    fn json(&self) -> Result<String> {
        serde_json::to_string(&self.info()).map_err(|e| anyhow!(e))
    }
}

pub fn show(cfg: &config::Local, format: print::Format, name: String) -> Result<String> {
    let chain = match cfg.chain(&name) {
        Some(chain) => chain,
        None => return Err(anyhow!("no such remote '{name}'.")),
    };

    print_with_format(chain, format)
}

pub fn list(cfg: &config::Local) -> Result<String> {
    let chains: Vec<String> = cfg.chains().keys().cloned().collect();
    if chains.is_empty() {
        Ok("".into())
    } else {
        let output: Vec<String> = chains
            .iter()
            .map(|key| (key.to_owned(), cfg.chain(key.as_str()).unwrap()))
            .map(|(name, chain)| {
                if log_enabled!(Level::Warn) {
                    format!("{name: <20}\t{url}", url = chain.url())
                } else {
                    name
                }
            })
            .collect();
        Ok(output.join("\n"))
    }
}
