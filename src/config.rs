//! Configuration management for TrinityChain

use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub miner: MinerConfig,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub p2p_port: u16,
    pub api_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct MinerConfig {
    pub threads: usize,
    pub beneficiary_address: String,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}
