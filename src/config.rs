use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::stats::StatId;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub cache: CacheConfig,
}

#[derive(Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_ttl")]
    pub ttl_minutes: u32,
}

fn default_ttl() -> u32 {
    15
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            ttl_minutes: default_ttl(),
        }
    }
}

#[derive(Deserialize, Default)]
pub struct GlyphConfig {
    #[serde(default = "default_glyph")]
    pub glyph: String,
}

fn default_glyph() -> String {
    ": ".to_string()
}

#[derive(Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_stats")]
    pub stats: Vec<StatId>,

    #[serde(default = "default_ascii")]
    pub ascii: String,

    #[serde(default = "default_ascii_color")]
    pub ascii_color: String,

    #[serde(default)]
    pub glyph: GlyphConfig,
}

fn default_ascii() -> String {
    "PACMAN_DEFAULT".to_string()
}

fn default_ascii_color() -> String {
    "yellow".to_string()
}

fn default_stats() -> Vec<StatId> {
    vec![
        StatId::Installed,
        StatId::Upgradable,
        StatId::LastUpdate,
        StatId::DownloadSize,
        StatId::InstalledSize,
        StatId::NetUpgradeSize,
        StatId::OrphanedPackages,
        StatId::CacheSize,
        StatId::MirrorUrl,
        StatId::MirrorHealth,
    ]
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            stats: default_stats(),
            ascii: default_ascii(),
            ascii_color: default_ascii_color(),
            glyph: GlyphConfig::default(),
        }
    }
}

impl Config {
    /// Returns ~/.config/pacfetch/pacfetch.toml
    pub fn config_path() -> Option<PathBuf> {
        // Check if running via sudo - use original user's config
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            let user_home = PathBuf::from(format!("/home/{}", sudo_user));
            if user_home.exists() {
                return Some(user_home.join(".config/pacfetch/pacfetch.toml"));
            }
        }

        dirs::config_dir().map(|p| p.join("pacfetch").join("pacfetch.toml"))
    }

    /// Returns ~/.cache/pacfetch/sync/
    pub fn cache_dir() -> Option<PathBuf> {
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            let user_home = PathBuf::from(format!("/home/{}", sudo_user));
            if user_home.exists() {
                return Some(user_home.join(".cache/pacfetch/sync"));
            }
        }

        dirs::cache_dir().map(|p| p.join("pacfetch").join("sync"))
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
