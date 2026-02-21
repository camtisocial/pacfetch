use chrono::Local;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

/// Convert seconds to a human-readable duration string
pub fn normalize_duration(seconds: i64) -> String {
    if seconds < 60 {
        return format!("{} second{}", seconds, if seconds != 1 { "s" } else { "" });
    }

    if seconds < 3600 {
        let minutes = seconds / 60;
        return format!("{} minute{}", minutes, if minutes != 1 { "s" } else { "" });
    }

    if seconds < 86400 {
        let hours = seconds / 3600;
        return format!("{} hour{}", hours, if hours != 1 { "s" } else { "" });
    }

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;

    format!(
        "{} day{} {} hour{}",
        days,
        if days != 1 { "s" } else { "" },
        hours,
        if hours != 1 { "s" } else { "" }
    )
}

/// Create a spinner with the given message
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\x1b' {
            result.push(c);
            continue;
        }
        match chars.peek().copied() {
            Some('[') => {
                chars.next();
                for nc in chars.by_ref() {
                    if ('@'..='~').contains(&nc) {
                        break;
                    }
                }
            }
            Some(']') => {
                chars.next();
                while let Some(nc) = chars.next() {
                    if nc == '\x07' {
                        break;
                    }
                    if nc == '\x1b' {
                        if chars.peek() == Some(&'\\') {
                            chars.next();
                        }
                        break;
                    }
                }
            }
            Some(_) => {
                chars.next();
            }
            None => {}
        }
    }
    result
}

/// Expand ~ to the user's home directory, respecting SUDO_USER
pub fn expand_path(path: &str) -> String {
    if path.starts_with('~') {
        let home = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            PathBuf::from(format!("/home/{}", sudo_user))
        } else {
            dirs::home_dir().unwrap_or_default()
        };
        path.replacen('~', &home.to_string_lossy(), 1)
    } else {
        path.to_string()
    }
}

/// Check if running as root
pub fn is_root() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

fn log_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("pacfetch").join("pacfetch.log"))
}

/// Log errors
pub fn log_error(msg: &str, debug: bool) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("[{}] {}\n", timestamp, msg);

    if debug {
        eprint!("{}", log_line);
    }

    if let Some(path) = log_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
}
