mod ascii;

use crate::color::parse_color;
use crate::config::{Config, TitleAlign, TitleConfig, TitleStyle, TitleWidth};
use crate::pacman::PacmanStats;
use crate::stats::{StatId, StatIdOrTitle};
use crossterm::style::{Color::*, Stylize};
use std::io;

/// Calculate the minimum width needed for a title based on its style
fn title_min_width(config: &TitleConfig, text: &str) -> usize {
    match config.style {
        TitleStyle::Stacked => text.chars().count(),
        TitleStyle::Embedded => {
            let caps_width = config.left_cap.chars().count() + config.right_cap.chars().count();
            if text.is_empty() {
                caps_width + 1
            } else {
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

    let align = config.align.clone().unwrap_or(match config.style {
        TitleStyle::Stacked => TitleAlign::Left,
        TitleStyle::Embedded => TitleAlign::Center,
    });

    match config.style {
        TitleStyle::Stacked => {
            if !text.is_empty() {
                let text_line = align_text(text, width, &align);
                let colored_text = match title_color {
                    Some(color) => format!("{}", text_line.bold().with(color)),
                    None => format!("{}", text_line.bold()),
                };
                lines.push(colored_text);
            }

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

            let full_line = if text.is_empty() {
                let line_str = repeat_pattern(&config.line, inner_width);
                let colored_line = match line_color {
                    Some(color) => format!("{}", line_str.with(color)),
                    None => line_str,
                };
                let colored_left_cap = match line_color {
                    Some(color) => format!("{}", left_cap.clone().with(color)),
                    None => left_cap.clone(),
                };
                let colored_right_cap = match line_color {
                    Some(color) => format!("{}", right_cap.clone().with(color)),
                    None => right_cap.clone(),
                };
                format!("{}{}{}", colored_left_cap, colored_line, colored_right_cap)
            } else {
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

                // Apply colors to each part separately to avoid ANSI reset interference
                let colored_left_cap = match line_color {
                    Some(color) => format!("{}", left_cap.clone().with(color)),
                    None => left_cap.clone(),
                };
                let colored_left_line = match line_color {
                    Some(color) => format!("{}", left_line.with(color)),
                    None => left_line,
                };
                let colored_text = match title_color {
                    Some(color) => format!("{}", text_with_spaces.bold().with(color)),
                    None => format!("{}", text_with_spaces.bold()),
                };
                let colored_right_line = match line_color {
                    Some(color) => format!("{}", right_line.with(color)),
                    None => right_line,
                };
                let colored_right_cap = match line_color {
                    Some(color) => format!("{}", right_cap.clone().with(color)),
                    None => right_cap.clone(),
                };

                format!(
                    "{}{}{}{}{}",
                    colored_left_cap,
                    colored_left_line,
                    colored_text,
                    colored_right_line,
                    colored_right_cap
                )
            };

            lines.push(full_line);
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

pub fn display_stats(stats: &PacmanStats, config: &Config) {
    let glyph = &config.display.glyph.glyph;
    let parsed_stats = config.display.parsed_stats();

    for stat_ref in &parsed_stats {
        match stat_ref {
            StatIdOrTitle::LegacyTitle => {
                crate::log::warn(
                    "Deprecated: [display.title] config. Use [display.titles.{name}] instead.",
                );
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
                        TitleWidth::Named(_) => title_text.chars().count().max(1),
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

pub fn display_stats_with_graphics(stats: &PacmanStats, config: &Config) -> io::Result<()> {
    let ascii_art = ascii::get_art(&config.display.ascii);
    let ascii_color = parse_color(&config.display.ascii_color);
    let glyph = &config.display.glyph.glyph;
    let parsed_stats = config.display.parsed_stats();

    // === PASS 1: Calculate content width ===
    let mut content_max_width: usize = 0;
    let mut title_entries: Vec<(String, TitleConfig, String)> = Vec::new(); // (name, config, resolved_text)
    let mut stat_lines_raw: Vec<(StatId, String)> = Vec::new(); // (stat_id, raw_line for width calc)
    let mut content_padding: usize = 0; // Max padding from embedded content titles

    for stat_ref in &parsed_stats {
        match stat_ref {
            StatIdOrTitle::LegacyTitle => {
                crate::log::warn(
                    "Deprecated: [display.title] config. Use [display.titles.{name}] instead.",
                );
                let title_text = resolve_title_text(&config.display.title, &stats.pacman_version);
                let min_width = title_min_width(&config.display.title, &title_text);
                content_max_width = content_max_width.max(min_width);
                // Track padding from titles with content width
                if matches!(&config.display.title.width, TitleWidth::Named(s) if s == "content") {
                    content_padding = content_padding.max(config.display.title.padding);
                }
                title_entries.push((
                    "__legacy__".to_string(),
                    config.display.title.clone(),
                    title_text,
                ));
            }
            StatIdOrTitle::NamedTitle(name) => {
                if let Some(title_config) = config.display.titles.get(name) {
                    let title_text = resolve_title_text(title_config, &stats.pacman_version);
                    let min_width = title_min_width(title_config, &title_text);
                    content_max_width = content_max_width.max(min_width);
                    // Track padding from titles with content width
                    if matches!(&title_config.width, TitleWidth::Named(s) if s == "content") {
                        content_padding = content_padding.max(title_config.padding);
                    }
                    title_entries.push((name.clone(), title_config.clone(), title_text));
                } else {
                    crate::log::warn(&format!("Title '{}' not found in config", name));
                }
            }
            StatIdOrTitle::Stat(stat_id) => {
                let value = stat_id
                    .format_value(stats)
                    .unwrap_or_else(|| "-".to_string());
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
                        TitleWidth::Named(s) if s == "content" => {
                            // Add padding*2 to "frame" the content
                            // (padding for left indent + padding for right margin)
                            content_max_width + (content_padding * 2)
                        }
                        TitleWidth::Named(_) => title_text.chars().count().max(1),
                        TitleWidth::Fixed(w) => *w,
                    };

                    let title_color = parse_color(&title_config.text_color);
                    let line_color = parse_color(&title_config.line_color);
                    let rendered =
                        render_title(title_config, title_text, width, title_color, line_color);
                    stats_lines.extend(rendered);
                }
            }
            StatIdOrTitle::Stat(_stat_id) => {
                if stat_idx < stat_lines_raw.len() {
                    let (raw_stat_id, _) = &stat_lines_raw[stat_idx];
                    stat_idx += 1;

                    // Format with colors for display
                    let formatted = format_stat_with_colors(*raw_stat_id, stats, config, glyph);
                    // Indent stat lines based on content padding
                    if content_padding > 0 {
                        stats_lines.push(format!("{}{}", " ".repeat(content_padding), formatted));
                    } else {
                        stats_lines.push(formatted);
                    }
                }
            }
        }
    }

    // Add color palette rows
    stats_lines.push(String::new());
    let colors = [
        Black,
        DarkRed,
        DarkGreen,
        DarkYellow,
        DarkBlue,
        DarkMagenta,
        DarkCyan,
        Grey,
    ];
    let bright_colors = [DarkGrey, Red, Green, Yellow, Blue, Magenta, Cyan, White];
    let mut color_row_1 = String::new();
    for color in &colors {
        color_row_1.push_str(&format!("{}", "   ".on(*color)));
    }
    let mut color_row_2 = String::new();
    for color in &bright_colors {
        color_row_2.push_str(&format!("{}", "   ".on(*color)));
    }
    // Indent color palette based on content padding
    if content_padding > 0 {
        let indent = " ".repeat(content_padding);
        stats_lines.push(format!("{}{}", indent, color_row_1));
        stats_lines.push(format!("{}{}", indent, color_row_2));
    } else {
        stats_lines.push(color_row_1);
        stats_lines.push(color_row_2);
    }

    // Print with ASCII art
    println!();
    if ascii_art.is_empty() {
        for line in &stats_lines {
            println!("{}", line);
        }
    } else {
        let art_width = ascii_art
            .iter()
            .map(|s| s.chars().count())
            .max()
            .unwrap_or(0);
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

// Helper to format a stat with colors
fn format_stat_with_colors(
    stat_id: StatId,
    stats: &PacmanStats,
    config: &Config,
    glyph: &str,
) -> String {
    if stat_id == StatId::MirrorHealth {
        match (&stats.mirror_url, stats.mirror_sync_age_hours) {
            (Some(_), Some(age)) => {
                let label = StatId::MirrorHealth.label();
                format!(
                    "{}{}{} (last sync {:.1} hours)",
                    label.bold().with(Yellow),
                    glyph,
                    "OK".green(),
                    age
                )
            }
            (Some(_), None) => {
                let label = StatId::MirrorHealth.label();
                format!(
                    "{}{}{} - could not check sync status",
                    label.bold().with(Yellow),
                    glyph,
                    "Err".red()
                )
            }
            (None, _) => {
                let label = StatId::MirrorHealth.label();
                format!(
                    "{}{}{} - no mirror found",
                    label.bold().with(Yellow),
                    glyph,
                    "Err".red()
                )
            }
        }
    } else if stat_id == StatId::Disk {
        if let (Some(used), Some(total)) = (stats.disk_used_bytes, stats.disk_total_bytes) {
            let used_gib = used as f64 / 1073741824.0;
            let total_gib = total as f64 / 1073741824.0;
            let pct = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            let pct_str = format!("({:.0}%)", pct);
            let colored_pct = if pct > 90.0 {
                format!("{}", pct_str.red())
            } else if pct >= 70.0 {
                format!("{}", pct_str.yellow())
            } else {
                format!("{}", pct_str.green())
            };
            let label = format!("Disk ({})", config.disk.path);
            format!(
                "{}{}{:.2} GiB / {:.2} GiB {}",
                label.bold().with(Yellow),
                glyph,
                used_gib,
                total_gib,
                colored_pct
            )
        } else {
            let label = format!("Disk ({})", config.disk.path);
            format!("{}{}-", label.bold().with(Yellow), glyph)
        }
    } else {
        let label = stat_id.label();
        let value = stat_id
            .format_value(stats)
            .unwrap_or_else(|| "-".to_string());
        format!("{}{}{}", label.bold().with(Yellow), glyph, value)
    }
}
