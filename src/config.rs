use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::stats::{self, StatId};

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "stats::default_stats")]
    pub stats: Vec<StatId>,

    #[serde(default = "default_ascii")]
    pub ascii: String,
}

fn default_ascii() -> String {
    "PACMAN_DEFAULT".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            display: DisplayConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            stats: stats::default_stats(),
            ascii: default_ascii(),
        }
    }
}

impl Config {
    /// Returns ~/.config/pacfetch/pacfetch.toml
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("pacfetch").join("pacfetch.toml"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Config::default();
        };

        let Ok(contents) = fs::read_to_string(&path) else {
            return Config::default();
        };

        toml::from_str(&contents).unwrap_or_default()
    }
}
