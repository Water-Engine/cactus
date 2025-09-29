use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct CactusConfig {
    pub tournament: Tournament,
    pub engine1: Engine,
    pub engine2: Engine,
}

#[derive(Debug, Deserialize)]
pub struct Tournament {
    pub name: String,
    pub rounds: u32,
    pub time_control: String,
}

#[derive(Debug, Deserialize)]
pub struct Engine {
    pub name: String,
    pub path: String,
}

pub fn parse_cactus_toml<P: AsRef<Path>>(
    path: P,
) -> Result<CactusConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: CactusConfig = toml::from_str(&content)?;
    Ok(config)
}
