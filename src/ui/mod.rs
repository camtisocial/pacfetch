mod ascii;

use crate::color::parse_color;
use crate::config::Config;
use crate::config::TitleConfig;
use crate::pacman::PacmanStats;
use crate::stats::StatId;
use crossterm::style::{Color::*, Stylize};
use std::io;

fn resolve_title_text(title_config: &TitleConfig, pacman_version: &Option<String>) -> String {
    match title_config.text.as_str() {
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
    for stat_id in &config.display.stats {
        if *stat_id == StatId::Title {
            let title_text = resolve_title_text(&config.display.title, &stats.pacman_version);
            let dashes = "-".repeat(title_text.len());
            println!("{}", title_text);
            println!("{}", dashes);
            continue;
        }

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

pub fn display_stats_with_graphics(stats: &PacmanStats, config: &Config) -> io::Result<()> {
    let ascii_art = ascii::get_art(&config.display.ascii);
    let ascii_color = parse_color(&config.display.ascii_color);

    // Build stat lines from config
    let mut stats_lines = vec![];

    let glyph = &config.display.glyph.glyph;
    for stat_id in &config.display.stats {
        if *stat_id == StatId::Title {
            let title_text = resolve_title_text(&config.display.title, &stats.pacman_version);
            let title_color = parse_color(&config.display.title.text_color);
            let line_color = parse_color(&config.display.title.line_color);
            let dashes = "-".repeat(title_text.len());

            let colored_title = match title_color {
                Some(color) => format!("{}", title_text.bold().with(color)),
                None => format!("{}", title_text.bold()),
            };
            stats_lines.push(colored_title);

            let colored_dashes = match line_color {
                Some(color) => format!("{}", dashes.with(color)),
                None => dashes,
            };
            stats_lines.push(colored_dashes);
            continue;
        }

        let value = stat_id
            .format_value(stats)
            .unwrap_or_else(|| "-".to_string());
        let formatted_value = if *stat_id == StatId::MirrorHealth {
            match (&stats.mirror_url, stats.mirror_sync_age_hours) {
                (Some(_), Some(age)) => format!("{} (last sync {:.1} hours)", "OK".green(), age),
                (Some(_), None) => format!("{} - could not check sync status", "Err".red()),
                (None, _) => format!("{} - no mirror found", "Err".red()),
            }
        } else if *stat_id == StatId::Disk {
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
                format!("{:.2} GiB / {:.2} GiB {}", used_gib, total_gib, colored_pct)
            } else {
                value
            }
        } else {
            value
        };
        let label = if *stat_id == StatId::Disk {
            format!("Disk ({})", config.disk.path)
        } else {
            stat_id.label().to_string()
        };
        stats_lines.push(format!(
            "{}{}{}",
            label.bold().with(Yellow),
            glyph,
            formatted_value
        ));
    }

    stats_lines.push(String::new());

    // color palette rows
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

    stats_lines.push(color_row_1);
    stats_lines.push(color_row_2);

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
