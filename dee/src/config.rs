use std::collections::HashMap;

use anyhow::{anyhow, Result};

use drand_core::chain::ChainInfo;
use serde::{Deserialize, Serialize};

pub type Chains = HashMap<String, ConfigChain>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Local {
    upstream: Option<String>,
    chains: Chains,
}

impl Local {
    const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
    const CONFIG_NAME: Option<&'static str> = Some("default");

    pub fn load() -> Self {
        confy::load(Self::APP_NAME, Self::CONFIG_NAME).unwrap()
    }

    pub fn path() -> Result<String> {
        confy::get_configuration_file_path(Self::APP_NAME, Self::CONFIG_NAME)
            .map(|path| path.to_str().unwrap().to_string())
            .map_err(|err| anyhow!(err))
    }

    pub fn store(&self) -> Result<()> {
        confy::store(Self::APP_NAME, Self::CONFIG_NAME, self).map_err(|err| anyhow!(err))
    }

    pub fn upstream(&self) -> Option<String> {
        self.upstream.clone()
    }

    pub fn upstream_chain(&self) -> Option<ConfigChain> {
        self.upstream().and_then(|u| self.chains.get(&u)).cloned()
    }

    pub fn chain(&self, name: &str) -> Option<ConfigChain> {
        self.chains.get(name).cloned()
    }

    pub fn chains(&self) -> Chains {
        self.chains.clone()
    }

    pub fn chain_by_hash(&self, hash: &[u8]) -> Option<(String, ConfigChain)> {
        self.chains
            .iter()
            .find(|(_, chain)| chain.info().hash() == hash)
            .map(|(name, chain)| (name.clone(), chain.clone()))
    }

    pub fn add_chain(&mut self, name: String, config_chain: ConfigChain) -> Result<()> {
        self.chains.insert(name.clone(), config_chain);
        if self.chains.len() == 1 {
            self.set_upstream(&name)
        } else {
            Ok(())
        }
    }

    pub fn remove_chain(&mut self, name: String) -> Result<()> {
        self.chains.remove(&name);
        Ok(())
    }

    pub fn rename_chain(&mut self, old: String, new: String) -> Result<()> {
        self.chains
            .remove(&old)
            .map(|v| {
                self.chains.insert(new, v);
            })
            .ok_or(anyhow!("no such remote '{old}'."))
    }

    pub fn set_url_chain(&mut self, name: String, url: &str) -> Result<()> {
        self.chains
            .get_mut(&name)
            .map(|v| {
                v.url = url.to_string();
            })
            .ok_or(anyhow!("no such remote '{name}'."))
    }

    pub fn set_upstream(&mut self, upstream: &str) -> Result<()> {
        self.chains
            .get(upstream)
            .map(|_| {
                self.upstream = Some(upstream.to_owned());
            })
            .ok_or(anyhow!("no such remote '{upstream}'."))
    }

    pub fn set_upstream_and_chain(&mut self, set_upstream: Option<String>) -> Result<ConfigChain> {
        let chain = set_upstream
            .map(|upstream| {
                self.set_upstream(&upstream).unwrap();
                upstream
            })
            .or(self.upstream());

        match chain {
            Some(chain) => Ok(self.chain(&chain).unwrap()),
            None => Err(anyhow!("No upstream")),
        }
    }
}

impl From<Local> for Option<&str> {
    fn from(_val: Local) -> Self {
        Some("default")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigChain {
    url: String,
    info: ChainInfo,
}

impl ConfigChain {
    pub fn new(url: &str, info: ChainInfo) -> Self {
        Self {
            url: url.to_string(),
            info,
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn info(&self) -> ChainInfo {
        self.info.clone()
    }
}
