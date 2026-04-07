use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Deserializer};

use crate::layout::Pos;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub layout: LayoutConfig,
    pub optimization: OptimizationConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LayoutConfig {
    pub finger_assignment: Vec<Vec<u8>>,
    pub finger_effort: Vec<Vec<f64>>,
    #[serde(deserialize_with = "deserialize_finger_home_positions")]
    pub finger_home_positions: HashMap<u8, Pos>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptimizationConfig {
    pub weights: Weights,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Weights {
    pub effort: f64,
}

impl From<Weights> for crate::optimizer::Weights {
    fn from(value: Weights) -> Self {
        Self {
            effort: value.effort,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

fn deserialize_finger_home_positions<'de, D>(deserializer: D) -> Result<HashMap<u8, Pos>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<u8, [usize; 2]> = HashMap::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .map(|(k, [row, col])| (k, Pos::new(row, col)))
        .collect())
}
