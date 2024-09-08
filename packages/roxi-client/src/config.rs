use crate::{error::ClientError, ClientResult};
use roxi_lib::types::SharedKey;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Secret {
    shared_key: SharedKey,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    host: String,
    port: String,
    shared_key: String,
}

impl Config {
    pub fn hostname(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl TryFrom<&PathBuf> for Config {
    type Error = ClientError;
    fn try_from(p: &PathBuf) -> ClientResult<Self> {
        let file = File::open(p)?;
        let content: serde_yaml::Value = serde_yaml::from_reader(file)?;
        let config: Config = serde_yaml::from_value(content)?;
        Ok(config)
    }
}

impl TryFrom<&Path> for Config {
    type Error = ClientError;
    fn try_from(p: &Path) -> ClientResult<Self> {
        let file = File::open(p)?;
        let content: serde_yaml::Value = serde_yaml::from_reader(file)?;
        let config: Config = serde_yaml::from_value(content)?;
        Ok(config)
    }
}

impl TryFrom<&str> for Config {
    type Error = ClientError;
    fn try_from(s: &str) -> ClientResult<Self> {
        let content: serde_yaml::Value = serde_yaml::from_str(s)?;
        let config: Config = serde_yaml::from_value(content)?;
        Ok(config)
    }
}
