use config::{Config, ConfigError, Environment, File};
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub debug: bool,
    pub annotation: String,
    pub rsa: Rsa,
    pub secrets: Secrets,
    pub volumes: Volumes,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rsa {
    pub bits: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Secrets {
    pub public_name: String,
    pub public_namespaces: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Volumes {
    pub mount: bool,
    pub public: Volume,
    pub private: Volume,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Volume {
    pub path: String,
}

impl Settings {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(path).required(true))?;
        s.merge(Environment::with_prefix("app"))?;
        s.try_into()
    }
}
