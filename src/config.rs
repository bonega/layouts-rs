use std::fs;
use std::path::Path;

use serde::{Deserialize, de::Error};

use crate::{
    layout,
    matrix::Matrix,
    optimizer::{SimulatedAnnealingConfig, Targets},
};

#[derive(Deserialize)]
pub struct Config {
    pub layout: layout::Config,
    pub optimization: OptimizationConfig,
}

#[derive(Deserialize, Clone)]
pub struct OptimizationConfig {
    pub targets: Targets,
    pub simulated_annealing: SimulatedAnnealingConfig,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

impl<'de, T> Deserialize<'de> for Matrix<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data: Vec<Vec<T>> = Vec::deserialize(deserializer)?;
        Self::new(data).map_err(D::Error::custom)
    }
}
