use anyhow::Result;

use crate::config;

pub fn config() -> Result<String> {
    config::Local::path()
}
