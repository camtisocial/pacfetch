use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::stats::{StatId, StatIdOrTitle};

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TitleStyle {
    #[default]
    Stacked,
    Embedded,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TitleWidth {
    Named(String),
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

#[derive(Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub default_args: String,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub disk: DiskConfig,
}

#[derive(Deserialize, Clone)]
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

#[derive(Deserialize, Clone)]
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

#[derive(Deserialize, Clone)]
pub struct PaletteConfig {
    #[serde(default = "default_palette_style")]
    pub style: String,
    #[serde(default = "default_palette_spacing")]
    pub spacing: usize,
}

fn default_palette_style() -> String {
    "blocks".to_string()
}

fn default_palette_spacing() -> usize {
    1
}

impl Default for PaletteConfig {
    fn default() -> Self {
        PaletteConfig {
            style: default_palette_style(),
            spacing: default_palette_spacing(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct GlyphConfig {
    #[serde(default = "default_glyph")]
    pub glyph: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub align: bool,
}

fn default_glyph() -> String {
    ": ".to_string()
}

impl Default for GlyphConfig {
    fn default() -> Self {
        GlyphConfig {
            glyph: default_glyph(),
            color: String::new(),
            align: false,
        }
    }
}

#[derive(Deserialize, Default, Clone)]
pub struct StatColorOverride {
    pub label: Option<String>,
    pub stat: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct ColorsConfig {
    #[serde(default = "default_label_color")]
    pub label: String,
    #[serde(default)]
    pub stat: String,
    #[serde(flatten)]
    pub overrides: HashMap<String, StatColorOverride>,
}

fn default_label_color() -> String {
    "bright_yellow".to_string()
}

impl Default for ColorsConfig {
    fn default() -> Self {
        ColorsConfig {
            label: default_label_color(),
            stat: String::new(),
            overrides: HashMap::new(),
        }
    }
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
    pub align: Option<TitleAlign>,

    #[serde(default = "default_line_char")]
    pub line: String,

    #[serde(default)]
    pub left_cap: String,

    #[serde(default)]
    pub right_cap: String,

    #[serde(default)]
    pub padding: usize,
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
            padding: 0,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct DisplayConfig {
    #[serde(default = "default_stats")]
    pub stats: Vec<String>,

    #[serde(default = "default_ascii")]
    pub ascii: String,

    #[serde(default = "default_ascii_color")]
    pub ascii_color: String,

    #[serde(default)]
    pub image: String,

    #[serde(default)]
    pub glyph: GlyphConfig,

    #[serde(default)]
    pub palette: PaletteConfig,

    #[serde(default)]
    pub colors: ColorsConfig,

    #[serde(default)]
    pub labels: HashMap<String, String>,

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

fn default_stats() -> Vec<String> {
    vec![
        "title".to_string(),
        "installed".to_string(),
        "upgradable".to_string(),
        "last_update".to_string(),
        "download_size".to_string(),
        "installed_size".to_string(),
        "net_upgrade_size".to_string(),
        "orphaned_packages".to_string(),
        "cache_size".to_string(),
        "disk".to_string(),
        "mirror_url".to_string(),
        "mirror_health".to_string(),
        "colors".to_string(),
    ]
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            stats: default_stats(),
            ascii: default_ascii(),
            ascii_color: default_ascii_color(),
            image: String::new(),
            glyph: GlyphConfig::default(),
            palette: PaletteConfig::default(),
            colors: ColorsConfig::default(),
            labels: HashMap::new(),
            title: TitleConfig::default(),
            titles: HashMap::new(),
        }
    }
}

impl DisplayConfig {
    pub fn parsed_stats(&self) -> Vec<StatIdOrTitle> {
        self.stats
            .iter()
            .filter_map(|s| match StatId::parse(s) {
                Ok(parsed) => Some(parsed),
                Err(e) => {
                    crate::log::warn(&e);
                    None
                }
            })
            .collect()
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

        // Migrate v1.0.0 configs that lack v1.1.0 sections
        let contents = if Self::needs_migration(&contents) {
            Self::migrate_config(&path, &contents).unwrap_or(contents)
        } else {
            contents
        };

        toml::from_str(&contents).unwrap_or_default()
    }

    /// v1.0.0 configs only had [display] with ascii + stats.
    /// Any config with v1.1.0 sections is already up to date.
    fn needs_migration(contents: &str) -> bool {
        !contents.contains("[display.glyph]")
            && !contents.contains("[display.titles")
            && !contents.contains("[display.palette]")
    }

    fn migrate_config(path: &PathBuf, contents: &str) -> Option<String> {
        // Full v1.0.0 → v1.1.0 migration
        let old: toml::Value = toml::from_str(contents).ok()?;

        let backup = path.with_extension("toml.bak");
        fs::copy(path, &backup).ok()?;

        let display = old.get("display");

        let ascii = display
            .and_then(|d| d.get("ascii"))
            .and_then(|v| v.as_str())
            .unwrap_or("PACMAN_DEFAULT");

        let mut stats: Vec<String> = display
            .and_then(|d| d.get("stats"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(default_stats);

        if !stats.iter().any(|s| s.starts_with("title")) {
            stats.insert(0, "title.header".to_string());
        }
        if !stats.iter().any(|s| s.starts_with("colors")) {
            stats.push("newline".to_string());
            stats.push("colors".to_string());
        }

        let stats_toml = {
            let entries: Vec<String> = stats.iter().map(|s| format!("    \"{}\"", s)).collect();
            format!("stats = [\n{},\n]", entries.join(",\n"))
        };

        let mut new_config = include_str!("../default_config.toml").to_string();

        new_config = new_config.replace(
            "ascii = \"PACMAN_DEFAULT\"",
            &format!("ascii = \"{}\"", ascii),
        );
        let stats_start = new_config.find("stats = [")?;
        let stats_end = new_config[stats_start..].find(']')? + stats_start + 1;
        new_config.replace_range(stats_start..stats_end, &stats_toml);

        fs::write(path, &new_config).ok()?;
        Some(new_config)
    }
}
