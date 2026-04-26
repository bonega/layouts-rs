use std::fs;
use std::path::Path;

use crate::{layout, optimizer::SimulatedAnnealingConfig, targets::Targets};

pub struct Config {
    pub layout: layout::Config,
    pub optimization: OptimizationConfig,
}

#[derive(Clone)]
pub struct OptimizationConfig {
    pub targets: Targets,
    pub simulated_annealing: SimulatedAnnealingConfig,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&content)?)
    }
}
