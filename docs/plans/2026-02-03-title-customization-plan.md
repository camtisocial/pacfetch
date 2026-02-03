# Title Customization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement configurable named titles with customizable width, style (stacked/embedded), alignment, and line characters.

**Architecture:** Titles become named entries in a HashMap under `[display.titles.{name}]`. The stats array references them via `title.{name}`. A two-pass rendering algorithm calculates max content width when `width = "content"`. Simple file logging captures warnings.

**Tech Stack:** Rust, serde (with HashMap for dynamic keys), crossterm (colors/styling), std::fs (logging)

---

## Task 1: Add Logging Module

**Files:**
- Create: `src/log.rs`
- Modify: `src/main.rs:1-6` (add mod declaration)

**Step 1: Create the logging module**

Create `src/log.rs`:

```rust
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn log_path() -> Option<PathBuf> {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let user_home = PathBuf::from(format!("/home/{}", sudo_user));
        if user_home.exists() {
            return Some(user_home.join(".cache/pacfetch/pacfetch.log"));
        }
    }
    dirs::cache_dir().map(|p| p.join("pacfetch").join("pacfetch.log"))
}

pub fn warn(msg: &str) {
    let Some(path) = log_path() else { return };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    else {
        return;
    };

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let _ = writeln!(file, "[{}] WARN: {}", timestamp, msg);
}
```

**Step 2: Add module declaration to main.rs**

In `src/main.rs`, add after line 1 (`mod color;`):

```rust
mod log;
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles with no errors

**Step 4: Commit**

```bash
git add src/log.rs src/main.rs
git commit -m "feat: add logging module for warnings"
```

---

## Task 2: Add New Config Types

**Files:**
- Modify: `src/config.rs`

**Step 1: Add new enums and structs**

Add these after the `use` statements in `src/config.rs` (around line 6):

```rust
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

// Note: Default depends on style, handled in rendering logic
```

**Step 2: Expand TitleConfig struct**

Replace the existing `TitleConfig` struct (around lines 63-73) with:

```rust
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

fn default_line_char() -> String {
    "-".to_string()
}
```

**Step 3: Update TitleConfig Default impl**

Replace the `Default` impl for `TitleConfig` (around lines 87-95) with:

```rust
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
```

**Step 4: Add use statement for HashMap**

At the top of `src/config.rs`, add to the `use std::...` line:

```rust
use std::collections::HashMap;
```

**Step 5: Add titles HashMap to DisplayConfig**

In the `DisplayConfig` struct (around line 97), add after the `title` field:

```rust
    #[serde(default)]
    pub titles: HashMap<String, TitleConfig>,
```

**Step 6: Verify compilation**

Run: `cargo check`
Expected: Compiles (may have warnings about unused fields)

**Step 7: Commit**

```bash
git add src/config.rs
git commit -m "feat: add new title config types (style, width, align, caps)"
```

---

## Task 3: Update StatId to Handle Named Titles

**Files:**
- Modify: `src/stats.rs`

**Step 1: Change Title variant handling**

The `StatId` enum uses serde's `rename_all = "snake_case"`. We need to handle `title.{name}` patterns specially. Since serde can't handle this directly with an enum, we'll keep `Title` for backwards compatibility and parse `title.xxx` strings manually.

Add a new function after the `StatId` enum (around line 22):

```rust
impl StatId {
    /// Parse a stat string, handling both regular stats and title.{name} references
    pub fn parse(s: &str) -> Result<StatIdOrTitle, String> {
        if let Some(name) = s.strip_prefix("title.") {
            return Ok(StatIdOrTitle::NamedTitle(name.to_string()));
        }

        // Handle legacy "title"
        if s == "title" {
            return Ok(StatIdOrTitle::LegacyTitle);
        }

        // Try to parse as regular StatId
        match s {
            "installed" => Ok(StatIdOrTitle::Stat(StatId::Installed)),
            "upgradable" => Ok(StatIdOrTitle::Stat(StatId::Upgradable)),
            "last_update" => Ok(StatIdOrTitle::Stat(StatId::LastUpdate)),
            "download_size" => Ok(StatIdOrTitle::Stat(StatId::DownloadSize)),
            "installed_size" => Ok(StatIdOrTitle::Stat(StatId::InstalledSize)),
            "net_upgrade_size" => Ok(StatIdOrTitle::Stat(StatId::NetUpgradeSize)),
            "orphaned_packages" => Ok(StatIdOrTitle::Stat(StatId::OrphanedPackages)),
            "cache_size" => Ok(StatIdOrTitle::Stat(StatId::CacheSize)),
            "mirror_url" => Ok(StatIdOrTitle::Stat(StatId::MirrorUrl)),
            "mirror_health" => Ok(StatIdOrTitle::Stat(StatId::MirrorHealth)),
            "disk" => Ok(StatIdOrTitle::Stat(StatId::Disk)),
            _ => Err(format!("unknown stat: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatIdOrTitle {
    Stat(StatId),
    NamedTitle(String),
    LegacyTitle,
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles with no errors

**Step 3: Commit**

```bash
git add src/stats.rs
git commit -m "feat: add StatIdOrTitle for parsing title.{name} references"
```

---

## Task 4: Update Config to Parse Stats with Named Titles

**Files:**
- Modify: `src/config.rs`

**Step 1: Add custom deserializer for stats**

Add after the imports in `src/config.rs`:

```rust
use crate::stats::{StatId, StatIdOrTitle};
```

**Step 2: Change stats field type in DisplayConfig**

In `DisplayConfig`, change the stats field from `Vec<StatId>` to `Vec<String>`:

```rust
    #[serde(default = "default_stats")]
    pub stats: Vec<String>,
```

**Step 3: Update default_stats function**

Update the `default_stats` function to return `Vec<String>`:

```rust
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
    ]
}
```

**Step 4: Add helper method to DisplayConfig**

Add an impl block for DisplayConfig after its Default impl:

```rust
impl DisplayConfig {
    /// Parse stats strings into StatIdOrTitle values
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
```

**Step 5: Verify compilation**

Run: `cargo check`
Expected: Compiles (may have errors in other files that use stats - we'll fix next)

**Step 6: Commit**

```bash
git add src/config.rs
git commit -m "feat: parse stats array as strings to support title.{name}"
```

---

## Task 5: Update stats.rs Helper Functions

**Files:**
- Modify: `src/stats.rs`

**Step 1: Update helper functions to work with parsed stats**

Update the `needs_*` functions to accept `&[StatIdOrTitle]`:

```rust
// --- stat fetch request helpers ---
pub fn needs_upgrade_stats(requested: &[StatIdOrTitle]) -> bool {
    requested.iter().any(|s| {
        matches!(
            s,
            StatIdOrTitle::Stat(StatId::Upgradable)
                | StatIdOrTitle::Stat(StatId::DownloadSize)
                | StatIdOrTitle::Stat(StatId::InstalledSize)
                | StatIdOrTitle::Stat(StatId::NetUpgradeSize)
        )
    })
}

pub fn needs_orphan_stats(requested: &[StatIdOrTitle]) -> bool {
    requested.iter().any(|s| matches!(s, StatIdOrTitle::Stat(StatId::OrphanedPackages)))
}

pub fn needs_mirror_health(requested: &[StatIdOrTitle]) -> bool {
    requested.iter().any(|s| matches!(s, StatIdOrTitle::Stat(StatId::MirrorHealth)))
}

pub fn needs_mirror_url(requested: &[StatIdOrTitle]) -> bool {
    requested.iter().any(|s| {
        matches!(s, StatIdOrTitle::Stat(StatId::MirrorUrl)) || matches!(s, StatIdOrTitle::Stat(StatId::MirrorHealth))
    })
}

pub fn needs_disk_stat(requested: &[StatIdOrTitle]) -> bool {
    requested.iter().any(|s| matches!(s, StatIdOrTitle::Stat(StatId::Disk)))
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Errors in pacman.rs and main.rs (we'll fix next)

**Step 3: Commit**

```bash
git add src/stats.rs
git commit -m "refactor: update needs_* helpers to use StatIdOrTitle"
```

---

## Task 6: Update pacman.rs to Use Parsed Stats

**Files:**
- Modify: `src/pacman.rs`

**Step 1: Find and update get_stats function signature**

First, read the relevant parts of pacman.rs to understand the current signature:

The `get_stats` function takes `&[StatId]`. We need to change it to work with `&[StatIdOrTitle]`.

Update the import at the top of `src/pacman.rs`:

```rust
use crate::stats::{
    needs_disk_stat, needs_mirror_health, needs_mirror_url, needs_orphan_stats,
    needs_upgrade_stats, StatIdOrTitle,
};
```

**Step 2: Update get_stats signature**

Change the function signature from:
```rust
pub fn get_stats(
    requested: &[StatId],
    ...
```

To:
```rust
pub fn get_stats(
    requested: &[StatIdOrTitle],
    ...
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Errors in main.rs (we'll fix next)

**Step 4: Commit**

```bash
git add src/pacman.rs
git commit -m "refactor: update get_stats to accept StatIdOrTitle slice"
```

---

## Task 7: Update main.rs to Use Parsed Stats

**Files:**
- Modify: `src/main.rs`

**Step 1: Update get_stats calls**

In `main.rs`, the `get_stats` calls pass `&config.display.stats`. Change these to use the parsed version.

Find all occurrences of `&config.display.stats` being passed to `get_stats` and change to:

```rust
&config.display.parsed_stats()
```

There should be approximately 3 occurrences (around lines 130, 141, 144).

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles, possibly with warnings in ui/mod.rs

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "refactor: use parsed_stats() in main.rs"
```

---

## Task 8: Implement Title Rendering Logic

**Files:**
- Modify: `src/ui/mod.rs`

This is the largest task. We'll implement the two-pass rendering algorithm.

**Step 1: Add new imports and helper struct**

At the top of `src/ui/mod.rs`, update imports:

```rust
mod ascii;

use crate::color::parse_color;
use crate::config::{Config, TitleAlign, TitleConfig, TitleStyle, TitleWidth};
use crate::pacman::PacmanStats;
use crate::stats::{StatId, StatIdOrTitle};
use crossterm::style::{Color::*, Stylize};
use std::io;
```

**Step 2: Add title rendering helper functions**

Add after the imports:

```rust
/// Calculate the minimum width needed for a title based on its style
fn title_min_width(config: &TitleConfig, text: &str) -> usize {
    match config.style {
        TitleStyle::Stacked => text.chars().count(),
        TitleStyle::Embedded => {
            let caps_width = config.left_cap.chars().count() + config.right_cap.chars().count();
            if text.is_empty() {
                // Just caps + at least one line char
                caps_width + 1
            } else {
                // caps + space + text + space + minimum line chars
                caps_width + 2 + text.chars().count() + 2
            }
        }
    }
}

/// Render a title line based on its configuration
fn render_title(
    config: &TitleConfig,
    text: &str,
    width: usize,
    title_color: Option<crossterm::style::Color>,
    line_color: Option<crossterm::style::Color>,
) -> Vec<String> {
    let mut lines = Vec::new();

    let align = config.align.clone().unwrap_or_else(|| {
        match config.style {
            TitleStyle::Stacked => TitleAlign::Left,
            TitleStyle::Embedded => TitleAlign::Center,
        }
    });

    match config.style {
        TitleStyle::Stacked => {
            // Text line
            if !text.is_empty() {
                let text_line = align_text(text, width, &align);
                let colored_text = match title_color {
                    Some(color) => format!("{}", text_line.bold().with(color)),
                    None => format!("{}", text_line.bold()),
                };
                lines.push(colored_text);
            }

            // Line below
            let line_str = repeat_pattern(&config.line, width);
            let colored_line = match line_color {
                Some(color) => format!("{}", line_str.with(color)),
                None => line_str,
            };
            lines.push(colored_line);
        }
        TitleStyle::Embedded => {
            let left_cap = &config.left_cap;
            let right_cap = &config.right_cap;
            let caps_width = left_cap.chars().count() + right_cap.chars().count();
            let inner_width = width.saturating_sub(caps_width);

            let line_content = if text.is_empty() {
                // Just line chars
                repeat_pattern(&config.line, inner_width)
            } else {
                // Line chars with text embedded
                let text_with_spaces = format!(" {} ", text);
                let text_len = text_with_spaces.chars().count();
                let remaining = inner_width.saturating_sub(text_len);

                let (left_len, right_len) = match align {
                    TitleAlign::Left => (1, remaining.saturating_sub(1)),
                    TitleAlign::Center => (remaining / 2, remaining - remaining / 2),
                    TitleAlign::Right => (remaining.saturating_sub(1), 1),
                };

                let left_line = repeat_pattern(&config.line, left_len);
                let right_line = repeat_pattern(&config.line, right_len);

                // Apply color to text portion
                let colored_text = match title_color {
                    Some(color) => format!("{}", text_with_spaces.bold().with(color)),
                    None => format!("{}", text_with_spaces.bold()),
                };

                format!("{}{}{}", left_line, colored_text, right_line)
            };

            // Combine with caps and apply line color
            let full_line = format!("{}{}{}", left_cap, line_content, right_cap);
            let colored_line = match line_color {
                Some(color) => format!("{}", full_line.with(color)),
                None => full_line,
            };
            lines.push(colored_line);
        }
    }

    lines
}

fn align_text(text: &str, width: usize, align: &TitleAlign) -> String {
    let text_len = text.chars().count();
    if text_len >= width {
        return text.to_string();
    }

    let padding = width - text_len;
    match align {
        TitleAlign::Left => format!("{}{}", text, " ".repeat(padding)),
        TitleAlign::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        }
        TitleAlign::Right => format!("{}{}", " ".repeat(padding), text),
    }
}

fn repeat_pattern(pattern: &str, width: usize) -> String {
    if pattern.is_empty() {
        return "-".repeat(width);
    }

    let pattern_len = pattern.chars().count();
    let repeats = (width / pattern_len) + 1;
    let full: String = pattern.repeat(repeats);
    full.chars().take(width).collect()
}
```

**Step 3: Update resolve_title_text to be public and handle TitleConfig**

Update the existing `resolve_title_text` function:

```rust
pub fn resolve_title_text(title_config: &TitleConfig, pacman_version: &Option<String>) -> String {
    match title_config.text.as_str() {
        "" => String::new(),
        "default" => pacman_version
            .clone()
            .unwrap_or_else(|| format!("pacfetch {}", env!("CARGO_PKG_VERSION"))),
        "pacman_ver" => {
            if let Some(full) = pacman_version {
                if let Some(dash_pos) = full.find(" - ") {
                    full[..dash_pos].trim().to_string()
                } else {
                    full.clone()
                }
            } else {
                "Pacman".to_string()
            }
        }
        "pacfetch_ver" => format!("pacfetch {}", env!("CARGO_PKG_VERSION")),
        custom => custom.to_string(),
    }
}
```

**Step 4: Verify compilation**

Run: `cargo check`
Expected: Compiles with warnings about unused functions

**Step 5: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat: add title rendering helper functions"
```

---

## Task 9: Update display_stats Function

**Files:**
- Modify: `src/ui/mod.rs`

**Step 1: Rewrite display_stats for named titles**

Replace the `display_stats` function:

```rust
pub fn display_stats(stats: &PacmanStats, config: &Config) {
    let glyph = &config.display.glyph.glyph;
    let parsed_stats = config.display.parsed_stats();

    for stat_ref in &parsed_stats {
        match stat_ref {
            StatIdOrTitle::LegacyTitle => {
                // Use old [display.title] config
                crate::log::warn("Deprecated: [display.title] config. Use [display.titles.{name}] instead.");
                let title_text = resolve_title_text(&config.display.title, &stats.pacman_version);
                let dashes = "-".repeat(title_text.chars().count());
                println!("{}", title_text);
                println!("{}", dashes);
            }
            StatIdOrTitle::NamedTitle(name) => {
                if let Some(title_config) = config.display.titles.get(name) {
                    let title_text = resolve_title_text(title_config, &stats.pacman_version);
                    let width = match &title_config.width {
                        TitleWidth::Named(s) if s == "title" => title_text.chars().count().max(1),
                        TitleWidth::Named(_) => title_text.chars().count().max(1), // "content" simplified for debug
                        TitleWidth::Fixed(w) => *w,
                    };
                    let title_lines = render_title(title_config, &title_text, width, None, None);
                    for line in title_lines {
                        println!("{}", line);
                    }
                } else {
                    crate::log::warn(&format!("Title '{}' not found in config", name));
                }
            }
            StatIdOrTitle::Stat(stat_id) => {
                if let Some(value) = stat_id.format_value(stats) {
                    let label = if *stat_id == StatId::Disk {
                        format!("Disk ({})", config.disk.path)
                    } else {
                        stat_id.label().to_string()
                    };
                    println!("{}{}{}", label, glyph, value);
                }
            }
        }
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles

**Step 3: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat: update display_stats for named titles"
```

---

## Task 10: Update display_stats_with_graphics (Two-Pass Rendering)

**Files:**
- Modify: `src/ui/mod.rs`

**Step 1: Implement two-pass rendering**

Replace the `display_stats_with_graphics` function with the full two-pass implementation:

```rust
pub fn display_stats_with_graphics(stats: &PacmanStats, config: &Config) -> io::Result<()> {
    let ascii_art = ascii::get_art(&config.display.ascii);
    let ascii_color = parse_color(&config.display.ascii_color);
    let glyph = &config.display.glyph.glyph;
    let parsed_stats = config.display.parsed_stats();

    // === PASS 1: Calculate content width ===
    let mut content_max_width: usize = 0;
    let mut title_entries: Vec<(String, TitleConfig, String)> = Vec::new(); // (name, config, resolved_text)
    let mut stat_lines_raw: Vec<(StatId, String)> = Vec::new(); // (stat_id, formatted_line)

    for stat_ref in &parsed_stats {
        match stat_ref {
            StatIdOrTitle::LegacyTitle => {
                crate::log::warn("Deprecated: [display.title] config. Use [display.titles.{name}] instead.");
                let title_text = resolve_title_text(&config.display.title, &stats.pacman_version);
                let min_width = title_min_width(&config.display.title, &title_text);
                content_max_width = content_max_width.max(min_width);
                title_entries.push(("__legacy__".to_string(), config.display.title.clone(), title_text));
            }
            StatIdOrTitle::NamedTitle(name) => {
                if let Some(title_config) = config.display.titles.get(name) {
                    let title_text = resolve_title_text(title_config, &stats.pacman_version);
                    let min_width = title_min_width(title_config, &title_text);
                    content_max_width = content_max_width.max(min_width);
                    title_entries.push((name.clone(), title_config.clone(), title_text));
                } else {
                    crate::log::warn(&format!("Title '{}' not found in config", name));
                }
            }
            StatIdOrTitle::Stat(stat_id) => {
                let value = stat_id.format_value(stats).unwrap_or_else(|| "-".to_string());
                let label = if *stat_id == StatId::Disk {
                    format!("Disk ({})", config.disk.path)
                } else {
                    stat_id.label().to_string()
                };
                let line = format!("{}{}{}", label, glyph, value);
                content_max_width = content_max_width.max(line.chars().count());
                stat_lines_raw.push((*stat_id, line));
            }
        }
    }

    // === PASS 2: Render at calculated widths ===
    let mut stats_lines: Vec<String> = Vec::new();
    let mut title_idx = 0;
    let mut stat_idx = 0;

    for stat_ref in &parsed_stats {
        match stat_ref {
            StatIdOrTitle::LegacyTitle | StatIdOrTitle::NamedTitle(_) => {
                if title_idx < title_entries.len() {
                    let (_, title_config, title_text) = &title_entries[title_idx];
                    title_idx += 1;

                    let width = match &title_config.width {
                        TitleWidth::Named(s) if s == "title" => title_text.chars().count().max(1),
                        TitleWidth::Named(s) if s == "content" => content_max_width,
                        TitleWidth::Named(_) => title_text.chars().count().max(1),
                        TitleWidth::Fixed(w) => *w,
                    };

                    let title_color = parse_color(&title_config.text_color);
                    let line_color = parse_color(&title_config.line_color);
                    let rendered = render_title(title_config, title_text, width, title_color, line_color);
                    stats_lines.extend(rendered);
                }
            }
            StatIdOrTitle::Stat(stat_id) => {
                if stat_idx < stat_lines_raw.len() {
                    let (raw_stat_id, raw_line) = &stat_lines_raw[stat_idx];
                    stat_idx += 1;

                    // Apply colors to specific stats
                    let formatted = if *raw_stat_id == StatId::MirrorHealth {
                        match (&stats.mirror_url, stats.mirror_sync_age_hours) {
                            (Some(_), Some(age)) => {
                                let label = format!("{}{}", StatId::MirrorHealth.label(), glyph);
                                format!("{}{} (last sync {:.1} hours)", label.bold().with(Yellow), "OK".green(), age)
                            }
                            (Some(_), None) => {
                                let label = format!("{}{}", StatId::MirrorHealth.label(), glyph);
                                format!("{}{} - could not check sync status", label.bold().with(Yellow), "Err".red())
                            }
                            (None, _) => {
                                let label = format!("{}{}", StatId::MirrorHealth.label(), glyph);
                                format!("{}{} - no mirror found", label.bold().with(Yellow), "Err".red())
                            }
                        }
                    } else if *raw_stat_id == StatId::Disk {
                        if let (Some(used), Some(total)) = (stats.disk_used_bytes, stats.disk_total_bytes) {
                            let used_gib = used as f64 / 1073741824.0;
                            let total_gib = total as f64 / 1073741824.0;
                            let pct = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
                            let pct_str = format!("({:.0}%)", pct);
                            let colored_pct = if pct > 90.0 {
                                format!("{}", pct_str.red())
                            } else if pct >= 70.0 {
                                format!("{}", pct_str.yellow())
                            } else {
                                format!("{}", pct_str.green())
                            };
                            let label = format!("Disk ({}){}", config.disk.path, glyph);
                            format!("{}{:.2} GiB / {:.2} GiB {}", label.bold().with(Yellow), used_gib, total_gib, colored_pct)
                        } else {
                            let label = format!("Disk ({}){}", config.disk.path, glyph);
                            format!("{}-", label.bold().with(Yellow))
                        }
                    } else {
                        let label = format!("{}{}", raw_stat_id.label(), glyph);
                        let value = raw_stat_id.format_value(stats).unwrap_or_else(|| "-".to_string());
                        format!("{}{}", label.bold().with(Yellow), value)
                    };
                    stats_lines.push(formatted);
                }
            }
        }
    }

    stats_lines.push(String::new());

    // Color palette rows
    let colors = [Black, DarkRed, DarkGreen, DarkYellow, DarkBlue, DarkMagenta, DarkCyan, Grey];
    let bright_colors = [DarkGrey, Red, Green, Yellow, Blue, Magenta, Cyan, White];

    let mut color_row_1 = String::new();
    for color in &colors {
        color_row_1.push_str(&format!("{}", "   ".on(*color)));
    }
    let mut color_row_2 = String::new();
    for color in &bright_colors {
        color_row_2.push_str(&format!("{}", "   ".on(*color)));
    }
    stats_lines.push(color_row_1);
    stats_lines.push(color_row_2);

    println!();

    if ascii_art.is_empty() {
        for line in &stats_lines {
            println!("{}", line);
        }
    } else {
        let art_width = ascii_art.iter().map(|s| s.chars().count()).max().unwrap_or(0);
        let padding = " ".repeat(art_width);
        let max_lines = ascii_art.len().max(stats_lines.len());

        for i in 0..max_lines {
            let art_line = ascii_art.get(i).map(|s| s.as_str()).unwrap_or(&padding);
            let stat_line = stats_lines.get(i).map(|s| s.as_str()).unwrap_or("");
            let colored_art = match ascii_color {
                Some(color) => format!("{}", art_line.with(color)),
                None => art_line.to_string(),
            };
            println!(" {}   {}", colored_art, stat_line);
        }
    }

    println!();
    Ok(())
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles

**Step 3: Test manually**

Run: `cargo run`
Expected: Should work with default config (legacy title)

**Step 4: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat: implement two-pass rendering for width=content"
```

---

## Task 11: Update default_config.toml

**Files:**
- Modify: `default_config.toml`

**Step 1: Update with new title format and documentation**

Replace `default_config.toml`:

```toml
################### DISPLAY ####################
[display]
## ascii options: "PACMAN_DEFAULT", "PACMAN_SMALL", "NONE", or a file path
ascii = "PACMAN_DEFAULT"
ascii_color = "yellow"

# Stats to display. Use "title.{name}" to reference titles defined below.
# Available stats: installed, upgradable, last_update, download_size, installed_size,
# net_upgrade_size, orphaned_packages, cache_size, disk, mirror_url, mirror_health
stats = [
    "title.header",
    "installed",
    "upgradable",
    "last_update",
    "download_size",
    "installed_size",
    "net_upgrade_size",
    "disk",
    "orphaned_packages",
    "cache_size",
    "mirror_url",
    "mirror_health",
]

[display.glyph]
glyph = ": "

################### TITLES ####################
# Define named titles under [display.titles.{name}]
# Reference them in the stats array as "title.{name}"
#
# Options:
#   text        = "default" | "pacman_ver" | "pacfetch_ver" | "" | custom string
#   text_color  = color name, hex (#RRGGBB), or "none"
#   line_color  = color name, hex (#RRGGBB), or "none"
#   style       = "stacked" (text above line) | "embedded" (text within line)
#   width       = "title" | "content" | integer
#   align       = "left" | "center" | "right" (default: left for stacked, center for embedded)
#   line        = line character(s), can be multi-char pattern like "=-"
#   left_cap    = left cap character(s) for embedded style
#   right_cap   = right cap character(s) for embedded style

[display.titles.header]
text = "default"
text_color = "bright_yellow"
line_color = "none"
style = "stacked"
width = "title"
line = "-"

# Example: divider with no text
# [display.titles.divider]
# text = ""
# style = "embedded"
# width = "content"
# line = "─"
# left_cap = "├"
# right_cap = "┤"

# Example: footer with custom text
# [display.titles.footer]
# text = "pacfetch"
# style = "embedded"
# width = "content"
# line = "─"
# left_cap = "╰"
# right_cap = "╯"

################### DISK ####################
[disk]
path = "/"

################### CACHE ####################
[cache]
# Set to 0 to always sync fresh
ttl_minutes = 15
```

**Step 2: Verify the app runs with new config**

Run: `cargo run`
Expected: Displays with the new title.header config

**Step 3: Commit**

```bash
git add default_config.toml
git commit -m "docs: update default_config.toml with named titles format"
```

---

## Task 12: Final Testing and Cleanup

**Step 1: Test various title configurations**

Create a test config at `~/.config/pacfetch/pacfetch.toml`:

```toml
[display]
ascii = "PACMAN_DEFAULT"
stats = [
    "title.header",
    "installed",
    "upgradable",
    "title.divider",
    "orphaned_packages",
    "cache_size",
    "title.footer",
]

[display.titles.header]
text = "default"
style = "stacked"
width = "content"
align = "center"

[display.titles.divider]
text = ""
style = "embedded"
width = "content"
line = "─"
left_cap = "├"
right_cap = "┤"

[display.titles.footer]
text = "pacfetch"
style = "embedded"
width = "content"
line = "─"
left_cap = "╰"
right_cap = "╯"
```

Run: `cargo run`
Expected: Three titles with matching widths, embedded styles with caps

**Step 2: Test edge cases**

- Test with missing title reference (should skip and log)
- Test with empty text (should show line only)
- Test with fixed width
- Test with multi-char line pattern

**Step 3: Run clippy**

Run: `cargo clippy`
Expected: No errors (warnings acceptable)

**Step 4: Format code**

Run: `cargo fmt`

**Step 5: Final commit**

```bash
git add -A
git commit -m "feat: complete title customization (issues #12, #13, #14)"
```

---

## Summary

This plan implements:
1. **Logging module** - Warnings to `~/.cache/pacfetch/pacfetch.log`
2. **New config types** - `TitleStyle`, `TitleWidth`, `TitleAlign` enums
3. **Named titles** - `HashMap<String, TitleConfig>` for `[display.titles.{name}]`
4. **Stats parsing** - `title.{name}` references in stats array
5. **Two-pass rendering** - Calculate max content width, then render
6. **Updated default config** - Documentation and examples
