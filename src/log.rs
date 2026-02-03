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
