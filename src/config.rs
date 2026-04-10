use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Deserializer, de::Error};

use crate::{
    matrix::{Matrix, Pos},
    optimizer::Targets,
};

#[derive(Deserialize)]
pub struct Config {
    pub layout: LayoutConfig,
    pub optimization: OptimizationConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LayoutConfig {
    pub finger_assignment: Matrix<u8>,
    pub finger_effort: Matrix<f64>,
    #[serde(deserialize_with = "deserialize_finger_home_positions")]
    pub finger_home_positions: HashMap<u8, Pos>,
}

#[derive(Deserialize)]
pub struct OptimizationConfig {
    pub targets: Targets,
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
