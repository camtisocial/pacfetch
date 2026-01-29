mod ascii;

use crate::color::parse_color;
use crate::config::Config;
use crate::pacman::PacmanStats;
use crate::stats::StatId;
use crossterm::style::{Color::*, Stylize};
use std::io;

pub fn display_stats(stats: &PacmanStats, config: &Config) {
    // Header
    if let Some(version) = &stats.pacman_version {
        let dashes = "-".repeat(version.len());
        println!("{}", version);
        println!("{}", dashes);
    } else {
        println!("----- pacfetch -----");
    }

    // stats
    for stat_id in &config.display.stats {
        if let Some(value) = stat_id.format_value(stats) {
            println!("{}: {}", stat_id.label(), value);
        }
    }
}

pub fn display_stats_with_graphics(stats: &PacmanStats, config: &Config) -> io::Result<()> {
    let ascii_art = ascii::get_art(&config.display.ascii);
    let ascii_color = parse_color(&config.display.ascii_color);

    // Build stat lines from config
    let mut stats_lines = vec![];

    if let Some(version) = &stats.pacman_version {
        let dashes = "-".repeat(version.len());
        stats_lines.push(format!("{}", version.as_str().bold().with(Yellow)));
        stats_lines.push(dashes);
    }

    // Add stats
    for stat_id in &config.display.stats {
        let value = stat_id
            .format_value(stats)
            .unwrap_or_else(|| "-".to_string());
        let formatted_value = if *stat_id == StatId::MirrorHealth {
            match (&stats.mirror_url, stats.mirror_sync_age_hours) {
                (Some(_), Some(age)) => format!("{} (last sync {:.1} hours)", "OK".green(), age),
                (Some(_), None) => format!("{} - could not check sync status", "Err".red()),
                (None, _) => format!("{} - no mirror found", "Err".red()),
            }
        } else {
            value
        };
        stats_lines.push(format!(
            "{}: {}",
            stat_id.label().bold().with(Yellow),
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
