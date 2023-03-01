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
    const APP_NAME: &str = env!("CARGO_PKG_NAME");
    const CONFIG_NAME: Option<&str> = Some("default");

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

    pub fn add_chain(&mut self, name: String, config_chain: ConfigChain) -> Result<()> {
        self.chains.insert(name, config_chain);
        Ok(())
    }

    pub fn remove_chain(&mut self, name: String) -> Result<()> {
        self.chains.remove(&name);
        Ok(())
    }

    pub fn rename_chain(&mut self, old: String, new: String) -> Result<()> {
        if let Some(v) = self.chains.remove(&old) {
            self.chains.insert(new, v);
            Ok(())
        } else {
            Err(anyhow!("Chain does not exist"))
        }
    }

    pub fn set_url_chain(&mut self, name: String, url: String) -> Result<()> {
        if let Some(v) = self.chains.get_mut(&name) {
            v.url = url;
            Ok(())
        } else {
            Err(anyhow!("Chain does not exist"))
        }
    }

    pub fn set_upstream(&mut self, upstream: &str) -> Result<()> {
        if self.chains.get(upstream).is_some() {
            self.upstream = Some(upstream.to_owned());
            Ok(())
        } else {
            Err(anyhow!("Chain does not exist"))
        }
    }

    pub fn set_upstream_and_chain(&mut self, set_upstream: Option<String>) -> Result<ConfigChain> {
        let chain = match set_upstream {
            Some(upstream) => {
                self.set_upstream(&upstream)?;
                upstream
            }
            None => match self.upstream() {
                Some(upstream) => upstream,
                None => return Err(anyhow!("No upstream")),
            },
        };
        Ok(self.chain(&chain).unwrap())
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
    pub fn new(url: String, info: ChainInfo) -> Self {
        Self { url, info }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn info(&self) -> ChainInfo {
        self.info.clone()
    }
}
