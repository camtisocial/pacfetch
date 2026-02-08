use std::fs;
use std::path::{Path, PathBuf};

use crate::util;

pub fn get_art(config: &str) -> Vec<String> {
    if config == "NONE" {
        return vec![];
    }

    // Raw art for things like cowsay
    if config.contains('\n') {
        let lines: Vec<String> = config.lines().map(|s| s.to_string()).collect();
        return normalize_width(lines);
    }

    // load from file
    if config.starts_with('/') || config.starts_with('~') || config.starts_with('.') {
        return normalize_width(load_from_file(config));
    }

    // built-ins
    match config {
        "PACMAN_DEFAULT" => PACMAN_DEFAULT.iter().map(|s| s.to_string()).collect(),
        "PACMAN_SMALL" => PACMAN_SMALL.iter().map(|s| s.to_string()).collect(),
        _ => PACMAN_DEFAULT.iter().map(|s| s.to_string()).collect(),
    }
}

fn normalize_width(lines: Vec<String>) -> Vec<String> {
    let max_width = lines
        .iter()
        .map(|s| util::strip_ansi(s).chars().count())
        .max()
        .unwrap_or(0);
    lines
        .into_iter()
        .map(|line| {
            let visible_width = util::strip_ansi(&line).chars().count();
            let padding = max_width - visible_width;
            if padding > 0 {
                format!("{}{}", line, " ".repeat(padding))
            } else {
                line
            }
        })
        .collect()
}

fn load_from_file(path: &str) -> Vec<String> {
    let expanded = if path.starts_with('~') {
        // When running via sudo, use the original user's home
        let home = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            PathBuf::from(format!("/home/{}", sudo_user))
        } else {
            dirs::home_dir().unwrap_or_default()
        };
        path.replacen('~', &home.to_string_lossy(), 1)
    } else {
        path.to_string()
    };

    let path = Path::new(&expanded);

    match fs::read_to_string(path) {
        Ok(contents) => contents.lines().map(|s| s.to_string()).collect(),
        Err(e) => {
            eprintln!("Failed to load ASCII art from '{}': {}", expanded, e);
            PACMAN_DEFAULT.iter().map(|s| s.to_string()).collect()
        }
    }
}

// --- Defaults ---

// Modified from https://emojicombos.com/pacman
const PACMAN_DEFAULT: [&str; 16] = [
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣤⣤⣤⣤⣤⣤⣤⣤⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⢀⣤⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡄  ⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⢠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⠛⠻⣿⣿⣿⣿⣿⣿⣿⣿⣆⠀⠀ ⠀⠀⠀⠀⠀⠀",
    "⠀⠀⣰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⡿⠃⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⣤⣴⣿⣿⣿⣿⣿⡿⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⢰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠋⠁⠀⠀⠀⣴⣿⣿⣿⣆⠀⠀⠀⣴⣿⣿⣿⣆",
    "⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣄⠀⠀⠀⠀⢿⣿⣿⣿⠏⠀⠀⠀⢿⣿⣿⣿⠏",
    "⠸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⣄⠀⠀⠉⠉⠁⠀ ⠀⠀⠀⠉⠉⠁⠀",
    "⠀⢻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡄⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠙⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠛⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠉⠻⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠙⠛⠛⠛⠛⠛⠛⠋⠉⠀⠀⠀⠀⠀ ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
];

// From pacman -v
const PACMAN_SMALL: [&str; 4] = [
    "  .--.                ",
    " / _.-' .-.  .-.  .-. ",
    " \\  '-. '-'  '-'  '-' ",
    "  '--'                ",
];
