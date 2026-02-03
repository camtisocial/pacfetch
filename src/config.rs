use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::stats::StatId;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TitleStyle {
    Stacked,
    Embedded,
}

impl Default for TitleStyle {
    fn default() -> Self {
        TitleStyle::Stacked
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TitleWidth {
    Named(String),  // "title" or "content"
    Fixed(usize),
}

impl Default for TitleWidth {
    fn default() -> Self {
        TitleWidth::Named("title".to_string())
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TitleAlign {
    Left,
    Center,
    Right,
}

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub disk: DiskConfig,
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

#[derive(Deserialize)]
pub struct DiskConfig {
    #[serde(default = "default_disk_path")]
    pub path: String,
}

fn default_disk_path() -> String {
    "/".to_string()
}

impl Default for DiskConfig {
    fn default() -> Self {
        DiskConfig {
            path: default_disk_path(),
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

#[derive(Debug, Clone, Deserialize)]
pub struct TitleConfig {
    #[serde(default = "default_title_text")]
    pub text: String,

    #[serde(default = "default_title_text_color")]
    pub text_color: String,

    #[serde(default = "default_title_line_color")]
    pub line_color: String,

    #[serde(default)]
    pub style: TitleStyle,

    #[serde(default)]
    pub width: TitleWidth,

    #[serde(default)]
    pub align: Option<TitleAlign>,  // None = use style default

    #[serde(default = "default_line_char")]
    pub line: String,

    #[serde(default)]
    pub left_cap: String,

    #[serde(default)]
    pub right_cap: String,
}

fn default_title_text() -> String {
    "default".to_string()
}

fn default_title_text_color() -> String {
    "bright_yellow".to_string()
}

fn default_title_line_color() -> String {
    "none".to_string()
}

fn default_line_char() -> String {
    "-".to_string()
}

impl Default for TitleConfig {
    fn default() -> Self {
        TitleConfig {
            text: default_title_text(),
            text_color: default_title_text_color(),
            line_color: default_title_line_color(),
            style: TitleStyle::default(),
            width: TitleWidth::default(),
            align: None,
            line: default_line_char(),
            left_cap: String::new(),
            right_cap: String::new(),
        }
    }
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

    #[serde(default)]
    pub title: TitleConfig,

    #[serde(default)]
    pub titles: HashMap<String, TitleConfig>,
}

fn default_ascii() -> String {
    "PACMAN_DEFAULT".to_string()
}

fn default_ascii_color() -> String {
    "yellow".to_string()
}

fn default_stats() -> Vec<StatId> {
    vec![
        StatId::Title,
        StatId::Installed,
        StatId::Upgradable,
        StatId::LastUpdate,
        StatId::DownloadSize,
        StatId::InstalledSize,
        StatId::NetUpgradeSize,
        StatId::OrphanedPackages,
        StatId::CacheSize,
        StatId::Disk,
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
            title: TitleConfig::default(),
            titles: HashMap::new(),
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
