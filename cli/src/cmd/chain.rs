use anyhow::{anyhow, Result};
use drand_client::chain::Chain;

use crate::config::{self, ConfigChain};

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

pub fn info(cfg: &config::Local, name: String) -> Result<String> {
    let chain = match cfg.chain(&name) {
        Some(chain) => chain,
        None => return Err(anyhow!("Chain does not exist")),
    };
    Ok(serde_json::to_string(&chain.info())?)
}

pub fn list(cfg: &config::Local) -> Result<String> {
    let chains: Vec<String> = cfg.chains().keys().cloned().collect();
    if chains.is_empty() {
        Ok("No chain".to_string())
    } else {
        Ok(chains.join("\n"))
    }
}
