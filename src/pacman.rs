use crate::stats::{
    needs_disk_stat, needs_mirror_health, needs_mirror_url, needs_orphan_stats,
    needs_upgrade_stats, StatId, StatIdOrTitle,
};
use crate::util;
use alpm::Alpm;
use chrono::{DateTime, FixedOffset, Local};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

const BYTES_PER_MIB: f64 = 1048576.0;

// --- Public data structures ---

#[derive(Debug, Default)]
pub struct PacmanStats {
    pub total_installed: u32,
    pub total_upgradable: u32,
    pub days_since_last_update: Option<i64>,
    pub download_size_mb: Option<f64>,
    pub total_installed_size_mb: Option<f64>,
    pub net_upgrade_size_mb: Option<f64>,
    pub orphaned_packages: Option<u32>,
    pub orphaned_size_mb: Option<f64>,
    pub cache_size_mb: Option<f64>,
    pub mirror_url: Option<String>,
    pub mirror_sync_age_hours: Option<f64>,
    pub pacman_version: Option<String>,
    pub disk_used_bytes: Option<u64>,
    pub disk_total_bytes: Option<u64>,
}

// --- Private helpers ---

#[derive(Default)]
struct UpgradeStats {
    download_size_mb: Option<f64>,
    installed_size_mb: Option<f64>,
    net_upgrade_size_mb: Option<f64>,
    package_count: u32,
}

#[derive(Clone, Copy)]
enum DbSyncState {
    Syncing(u8),
    Complete,
}

struct SyncProgress {
    core: DbSyncState,
    extra: DbSyncState,
    multilib: DbSyncState,
}

impl SyncProgress {
    fn new() -> Self {
        Self {
            core: DbSyncState::Syncing(0),
            extra: DbSyncState::Syncing(0),
            multilib: DbSyncState::Syncing(0),
        }
    }

    fn format(&self) -> String {
        format!(
            "core {} | extra {} | multilib {}",
            Self::format_state(self.core),
            Self::format_state(self.extra),
            Self::format_state(self.multilib)
        )
    }

    fn format_state(state: DbSyncState) -> String {
        match state {
            DbSyncState::Syncing(pct) => format!("{}%", pct),
            DbSyncState::Complete => "✓".to_string(),
        }
    }

    fn update_from_line(&mut self, line: &str) {
        let clean = util::strip_ansi(line);
        let trimmed = clean.trim();

        if trimmed.contains("is up to date") {
            if trimmed.starts_with("core") {
                self.core = DbSyncState::Complete;
            } else if trimmed.starts_with("extra") {
                self.extra = DbSyncState::Complete;
            } else if trimmed.starts_with("multilib") {
                self.multilib = DbSyncState::Complete;
            }
            return;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            let db_name = parts[0];
            let last = parts[parts.len() - 1];

            if let Some(pct_str) = last.strip_suffix('%')
                && let Ok(pct) = pct_str.parse::<u8>()
            {
                let state = if pct >= 100 {
                    DbSyncState::Complete
                } else {
                    DbSyncState::Syncing(pct)
                };

                match db_name {
                    "core" => self.core = state,
                    "extra" => self.extra = state,
                    "multilib" => self.multilib = state,
                    _ => {}
                }
            }
        }
    }
}

/// Copy modification time from src to dest using libc
fn copy_mtime(src: &std::path::Path, dest: &std::path::Path) {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::MetadataExt;

    let Ok(meta) = fs::metadata(src) else {
        return;
    };

    let mtime = libc::timespec {
        tv_sec: meta.mtime(),
        tv_nsec: meta.mtime_nsec(),
    };
    let atime = libc::timespec {
        tv_sec: 0,
        tv_nsec: libc::UTIME_OMIT,
    };
    let times = [atime, mtime];

    let path_cstr = std::ffi::CString::new(dest.as_os_str().as_bytes()).ok();
    if let Some(cstr) = path_cstr {
        unsafe {
            libc::utimensat(libc::AT_FDCWD, cstr.as_ptr(), times.as_ptr(), 0);
        }
    }
}

/// database cache at ~/.cache/pacfetch/
struct DbCache {
    path: PathBuf,
}

impl DbCache {
    /// Get or create the persistent cache directory
    fn new() -> Option<Self> {
        let cache_dir = crate::config::Config::cache_dir()?;
        let cache_path = cache_dir.parent()?; // ~/.cache/pacfetch/

        fs::create_dir_all(&cache_dir).ok()?;

        let local_link = cache_path.join("local");
        if !local_link.exists() {
            symlink("/var/lib/pacman/local", &local_link).ok()?;
        }

        Some(Self {
            path: cache_path.to_path_buf(),
        })
    }

    fn dbpath(&self) -> &str {
        self.path.to_str().unwrap_or("/tmp/pacfetch")
    }

    fn sync_dir(&self) -> PathBuf {
        self.path.join("sync")
    }

    fn is_fresh(&self, ttl_minutes: u32) -> bool {
        if ttl_minutes == 0 {
            return false;
        }

        let sync_dir = self.sync_dir();
        let required_dbs = ["core.db", "extra.db", "multilib.db"];

        for db in required_dbs {
            let db_path = sync_dir.join(db);
            let Ok(meta) = fs::metadata(&db_path) else {
                return false;
            };

            let Ok(modified) = meta.modified() else {
                return false;
            };

            let Ok(age) = modified.elapsed() else {
                return false;
            };

            if age.as_secs() > (ttl_minutes as u64 * 60) {
                return false;
            }
        }

        true
    }

    /// Copy system databases to cache
    fn copy_system_dbs(&self) {
        let sync_dir = self.sync_dir();
        let source_sync = PathBuf::from("/var/lib/pacman/sync");

        if !source_sync.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(&source_sync) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "db")
                    && let Some(filename) = path.file_name()
                {
                    let dest = sync_dir.join(filename);
                    // Only copy if dest doesn't exist or is older than source
                    let should_copy = match (fs::metadata(&path), fs::metadata(&dest)) {
                        (Ok(src_meta), Ok(dest_meta)) => {
                            src_meta.modified().ok() > dest_meta.modified().ok()
                        }
                        (Ok(_), Err(_)) => true,
                        _ => false,
                    };
                    if should_copy && fs::copy(&path, &dest).is_ok() {
                        copy_mtime(&path, &dest);
                    }
                }
            }
        }
    }

    /// Update mtime
    fn touch(&self) {
        use std::os::unix::ffi::OsStrExt;

        let now = libc::timespec {
            tv_sec: 0,
            tv_nsec: libc::UTIME_NOW,
        };
        let times = [now, now];

        let sync_dir = self.sync_dir();
        for db in ["core.db", "extra.db", "multilib.db"] {
            let db_path = sync_dir.join(db);
            if let Ok(cstr) = std::ffi::CString::new(db_path.as_os_str().as_bytes()) {
                unsafe {
                    libc::utimensat(libc::AT_FDCWD, cstr.as_ptr(), times.as_ptr(), 0);
                }
            }
        }
    }
}

fn calculate_upgrade_stats_with_sync(
    spinner: Option<&ProgressBar>,
    debug: bool,
    ttl_minutes: u32,
) -> UpgradeStats {
    let fail = UpgradeStats::default();

    let cache = match DbCache::new() {
        Some(c) => c,
        None => {
            util::log_error("Failed to create cache directory", debug);
            return fail;
        }
    };

    // Check if cache is fresh
    if cache.is_fresh(ttl_minutes) {
        if debug {
            eprintln!(
                "  Database sync: SKIP (cache fresh, TTL {}min)",
                ttl_minutes
            );
        }
        if let Some(pb) = spinner {
            pb.set_message("Using cached databases");
            std::thread::sleep(std::time::Duration::from_millis(100));
            pb.set_message("Gathering stats");
        }

        let calc_start = Instant::now();
        let stats = calculate_upgrade_stats(cache.dbpath(), debug);
        if debug {
            eprintln!("  Stats calculation: {:?}", calc_start.elapsed());
        }
        return stats;
    }

    // not fres
    cache.copy_system_dbs();

    let sync_start = Instant::now();

    let cmd = if util::is_root() {
        format!("pacman -Sy --dbpath {} --logfile /dev/null", cache.dbpath())
    } else {
        format!(
            "fakeroot -- pacman -Sy --disable-sandbox-filesystem --dbpath {} --logfile /dev/null",
            cache.dbpath()
        )
    };

    let mut session = match expectrl::spawn(&cmd) {
        Ok(s) => s,
        Err(e) => {
            util::log_error(&format!("Failed to spawn pacman: {}", e), debug);
            return fail;
        }
    };

    session.set_expect_timeout(Some(std::time::Duration::from_millis(100)));

    let mut progress = SyncProgress::new();
    if let Some(pb) = spinner {
        pb.set_message(format!("Syncing databases: {}", progress.format()));
    }

    let mut line_buffer = String::new();
    let mut sync_success = false;

    loop {
        match session.is_alive() {
            Ok(true) => {}
            Ok(false) => {
                if !line_buffer.is_empty() {
                    progress.update_from_line(&line_buffer);
                    if let Some(pb) = spinner {
                        pb.set_message(format!("Syncing databases: {}", progress.format()));
                    }
                }
                sync_success = true;
                break;
            }
            Err(_) => break,
        }

        let mut buf = [0u8; 1024];
        match session.try_read(&mut buf) {
            Ok(0) => continue,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);

                for ch in chunk.chars() {
                    if ch == '\n' || ch == '\r' {
                        if !line_buffer.is_empty() {
                            progress.update_from_line(&line_buffer);
                            if let Some(pb) = spinner {
                                pb.set_message(format!("Syncing databases: {}", progress.format()));
                            }
                        }
                        line_buffer.clear();
                    } else {
                        line_buffer.push(ch);
                    }
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                _ => break,
            },
        }
    }

    if !sync_success {
        util::log_error("Database sync failed or was interrupted", debug);
        return fail;
    }

    // Mark cache as fresh
    cache.touch();

    if debug {
        eprintln!("  Database sync: {:?}", sync_start.elapsed());
    }

    if let Some(pb) = spinner {
        progress.core = DbSyncState::Complete;
        progress.extra = DbSyncState::Complete;
        progress.multilib = DbSyncState::Complete;
        pb.set_message(format!("Syncing databases: {}", progress.format()));
        std::thread::sleep(std::time::Duration::from_millis(100));
        pb.set_message("Gathering stats");
    }

    let calc_start = Instant::now();
    let stats = calculate_upgrade_stats(cache.dbpath(), debug);
    if debug {
        eprintln!("  Stats calculation: {:?}", calc_start.elapsed());
    }
    stats
}

fn get_installed_count() -> u32 {
    let output = Command::new("pacman").arg("-Q").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().count() as u32
}

fn get_seconds_since_update() -> Option<i64> {
    let contents = fs::read_to_string("/var/log/pacman.log").expect("Failed to read pacman.log");

    let mut saw_upgrade_start = false;
    let mut upgrade_start_timestamp: Option<String> = None;
    let mut last_valid_timestamp: Option<String> = None;

    for line in contents.lines() {
        let trimmed = line.trim();

        let timestamp = trimmed
            .split(']')
            .next()
            .map(|x| x.trim_start_matches('['))
            .unwrap_or("");

        if trimmed.contains("starting full system upgrade") {
            saw_upgrade_start = true;
            upgrade_start_timestamp = Some(timestamp.to_string());
        }

        if saw_upgrade_start && trimmed.contains("transaction completed") {
            last_valid_timestamp = upgrade_start_timestamp.clone();
            saw_upgrade_start = false;
        }
    }

    if let Some(ts) = last_valid_timestamp {
        let formatted_date = format!("{}:{}", &ts[..22], &ts[22..]);

        let parsed: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(&formatted_date).unwrap();

        let last_update_local = parsed.with_timezone(&Local);
        let now = Local::now();
        let duration = now.signed_duration_since(last_update_local);
        let seconds = duration.num_seconds().max(0);

        return Some(seconds);
    }

    None
}

/// Calculate upgrade stats from a db path
fn calculate_upgrade_stats(dbpath: &str, debug: bool) -> UpgradeStats {
    let fail = UpgradeStats::default();

    let mut alpm = match Alpm::new("/", dbpath) {
        Ok(a) => a,
        Err(e) => {
            util::log_error(&format!("Failed to initialize alpm: {}", e), debug);
            return fail;
        }
    };

    let _ = alpm.register_syncdb_mut("core", alpm::SigLevel::NONE);
    let _ = alpm.register_syncdb_mut("extra", alpm::SigLevel::NONE);
    let _ = alpm.register_syncdb_mut("multilib", alpm::SigLevel::NONE);

    if let Err(e) = alpm.trans_init(alpm::TransFlag::NO_LOCK) {
        util::log_error(&format!("Failed to init transaction: {}", e), debug);
        return fail;
    }

    if let Err(e) = alpm.sync_sysupgrade(false) {
        let msg = format!("Failed to sync sysupgrade: {}", e);
        let _ = alpm.trans_release();
        util::log_error(&msg, debug);
        return fail;
    }

    let prepare_err = alpm.trans_prepare().err().map(|e| format!("{}", e));
    if let Some(msg) = prepare_err {
        let _ = alpm.trans_release();
        util::log_error(&format!("Failed to prepare transaction: {}", msg), debug);
        return fail;
    }

    let localdb = alpm.localdb();

    let mut total_download_size: i64 = 0;
    let mut total_installed_size: i64 = 0;
    let mut net_upgrade_size: i64 = 0;
    let mut package_count: u32 = 0;

    for pkg in alpm.trans_add().into_iter() {
        package_count += 1;
        total_download_size += pkg.download_size();
        let new_size = pkg.isize();
        total_installed_size += new_size;

        if let Ok(oldpkg) = localdb.pkg(pkg.name()) {
            let old_size = oldpkg.isize();
            net_upgrade_size += new_size - old_size;
        } else {
            net_upgrade_size += new_size;
        }
    }

    for pkg in alpm.trans_remove().into_iter() {
        net_upgrade_size -= pkg.isize();
    }

    let _ = alpm.trans_release();

    let download_mib = total_download_size as f64 / BYTES_PER_MIB;
    let installed_mib = total_installed_size as f64 / BYTES_PER_MIB;
    let mut net_mib = net_upgrade_size as f64 / BYTES_PER_MIB;

    if net_mib > -0.01 && net_mib < 0.01 {
        net_mib = 0.0;
    }

    UpgradeStats {
        download_size_mb: Some(download_mib),
        installed_size_mb: Some(installed_mib),
        net_upgrade_size_mb: Some(net_mib),
        package_count,
    }
}

fn get_orphaned_packages(debug: bool) -> (Option<u32>, Option<f64>) {
    let alpm = match Alpm::new("/", "/var/lib/pacman") {
        Ok(a) => a,
        Err(e) => {
            util::log_error(
                &format!("Failed to init alpm for orphan check: {}", e),
                debug,
            );
            return (None, None);
        }
    };

    let localdb = alpm.localdb();
    let mut count = 0;
    let mut total_size: i64 = 0;

    for pkg in localdb.pkgs().into_iter() {
        if pkg.reason() == alpm::PackageReason::Depend
            && pkg.required_by().is_empty()
            && pkg.optional_for().is_empty()
        {
            count += 1;
            total_size += pkg.isize();
        }
    }

    let size_mb = total_size as f64 / BYTES_PER_MIB;
    (Some(count), Some(size_mb))
}

fn get_cache_size() -> Option<f64> {
    let cache_path = std::path::Path::new("/var/cache/pacman/pkg");

    if let Ok(entries) = std::fs::read_dir(cache_path) {
        let total_size: u64 = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .map(|m| m.len())
            .sum();

        Some(total_size as f64 / BYTES_PER_MIB)
    } else {
        None
    }
}

fn expand_tilde(path: &str) -> String {
    if (path == "~" || path.starts_with("~/"))
        && let Some(home) = dirs::home_dir()
    {
        return path.replacen('~', &home.to_string_lossy(), 1);
    }
    path.to_string()
}

fn get_disk_usage(path: &str) -> Option<(u64, u64)> {
    use nix::sys::statvfs::statvfs;

    let expanded = expand_tilde(path);
    let stat = statvfs(expanded.as_str()).ok()?;
    let frsize = stat.fragment_size() as u64;
    let total = stat.blocks() * frsize;
    let used = (stat.blocks() - stat.blocks_free()) * frsize;
    Some((used, total))
}

fn get_mirror_url() -> Option<String> {
    let mirrorlist = fs::read_to_string("/etc/pacman.d/mirrorlist").ok()?;

    for line in mirrorlist.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Server = ") {
            let url = trimmed.strip_prefix("Server = ")?;
            let base_url = url.split("/$repo").next()?;
            return Some(base_url.to_string());
        }
    }
    None
}

fn get_pacman_version() -> Option<String> {
    let output = Command::new("pacman").arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.contains("Pacman v")
            && line.contains("libalpm v")
            && let Some(version_start) = line.find("Pacman v")
        {
            let version_str = &line[version_start..];
            return Some(version_str.trim().to_string());
        }
    }
    None
}

fn check_mirror_sync(mirror_url: &str, debug: bool) -> Option<f64> {
    let lastsync_url = format!("{}/lastsync", mirror_url);

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            util::log_error(&format!("Failed to build HTTP client: {}", e), debug);
            return None;
        }
    };

    // retry once
    let mut last_error = String::new();
    let response = (0..2).find_map(|attempt| match client.get(&lastsync_url).send() {
        Ok(r) => Some(r),
        Err(e) => {
            last_error = format!("{}", e);
            if attempt == 0 {
                util::log_error(
                    &format!("Failed to fetch {} (retrying): {}", lastsync_url, e),
                    debug,
                );
            }
            None
        }
    });

    let response = match response {
        Some(r) => r,
        None => {
            util::log_error(
                &format!(
                    "Failed to fetch {} after retry: {}",
                    lastsync_url, last_error
                ),
                debug,
            );
            return None;
        }
    };

    if !response.status().is_success() {
        util::log_error(
            &format!("Mirror returned status {}", response.status()),
            debug,
        );
        return None;
    }

    let timestamp_str = match response.text() {
        Ok(t) => t,
        Err(e) => {
            util::log_error(&format!("Failed to read response: {}", e), debug);
            return None;
        }
    };

    let timestamp: i64 = match timestamp_str.trim().parse() {
        Ok(t) => t,
        Err(e) => {
            util::log_error(
                &format!(
                    "Failed to parse timestamp '{}': {}",
                    timestamp_str.trim(),
                    e
                ),
                debug,
            );
            return None;
        }
    };

    let now = Local::now().timestamp();
    let age_seconds = now - timestamp;
    let age_hours = age_seconds as f64 / 3600.0;

    Some(age_hours.max(0.0))
}

fn filter_upgrade_line(line: &str) -> bool {
    let clean = util::strip_ansi(line);
    let trimmed = clean.trim();

    if trimmed.is_empty() {
        return false;
    }

    if trimmed.contains("Total Download Size:")
        || trimmed.contains("Total Installed Size:")
        || trimmed.contains("Net Upgrade Size:")
    {
        return false;
    }

    if trimmed.contains("resolving dependencies")
        || trimmed.contains("looking for conflicting packages")
        || trimmed.contains(":: Starting full system upgrade...")
    {
        return false;
    }

    true
}

fn should_print(line: &str, filter: bool) -> bool {
    if filter {
        filter_upgrade_line(line)
    } else {
        true
    }
}

fn run_pacman_pty(args: &[&str], filter: bool) -> Result<(), String> {
    use std::io::Write;

    let cmd = format!("pacman {}", args.join(" "));
    let mut session =
        expectrl::spawn(&cmd).map_err(|e| format!("Failed to spawn pacman: {}", e))?;

    if let Ok((cols, rows)) = crossterm::terminal::size() {
        let _ = session.get_process_mut().set_window_size(cols, rows);
    }

    session.set_expect_timeout(Some(std::time::Duration::from_millis(100)));

    let mut stdout = std::io::stdout();
    let mut line_buffer = String::new();
    let mut raw_mode = false;

    let mut process_exited = false;

    loop {
        if !process_exited {
            match session.is_alive() {
                Ok(true) => {}
                Ok(false) => process_exited = true,
                Err(_) => process_exited = true,
            }
        }

        let mut buf = [0u8; 1024];
        match session.try_read(&mut buf) {
            Ok(0) => {
                if process_exited {
                    break;
                }
                continue;
            }
            Ok(n) => {
                if raw_mode {
                    stdout.write_all(&buf[..n]).ok();
                    stdout.flush().ok();
                    continue;
                }

                let chunk = String::from_utf8_lossy(&buf[..n]);

                for ch in chunk.chars() {
                    if ch == '\n' {
                        if should_print(&line_buffer, filter) {
                            println!("{}", line_buffer);
                        }
                        line_buffer.clear();
                    } else if ch == '\r' {
                    } else {
                        line_buffer.push(ch);

                        if line_buffer.ends_with("[Y/n] ")
                            || (line_buffer.contains("::") && line_buffer.ends_with("]: "))
                        {
                            if should_print(&line_buffer, filter) {
                                if line_buffer.contains("Proceed with installation") {
                                    println!("\n");
                                }
                                print!("{}", line_buffer);
                                let _ = stdout.flush();
                            }
                            line_buffer.clear();

                            let mut input = String::new();
                            if std::io::stdin().read_line(&mut input).is_ok() {
                                let _ = session.send_line(input.trim());
                                raw_mode = true;
                            }
                        }
                    }
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted => {
                    if process_exited {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                _ => break,
            },
        }
    }

    if !line_buffer.is_empty() && should_print(&line_buffer, filter) {
        println!("{}", line_buffer);
    }

    print!("\x1b[0m");
    let _ = stdout.flush();

    Ok(())
}

fn run_pacman_sync() -> Result<(), String> {
    if !util::is_root() {
        return Err("you cannot perform this operation unless you are root.".to_string());
    }

    let mut session =
        expectrl::spawn("pacman -Sy").map_err(|e| format!("Failed to spawn pacman: {}", e))?;

    if let Ok((cols, rows)) = crossterm::terminal::size() {
        let _ = session.get_process_mut().set_window_size(cols, rows);
    }

    session.set_expect_timeout(Some(std::time::Duration::from_millis(100)));

    let mut progress = SyncProgress::new();
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} Syncing databases: {msg}")
            .unwrap(),
    );
    pb.set_message(progress.format());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut line_buffer = String::new();

    loop {
        match session.is_alive() {
            Ok(true) => {}
            Ok(false) => {
                if !line_buffer.is_empty() {
                    progress.update_from_line(&line_buffer);
                    pb.set_message(progress.format());
                }
                break;
            }
            Err(_) => break,
        }

        let mut buf = [0u8; 1024];
        match session.try_read(&mut buf) {
            Ok(0) => continue,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);

                for ch in chunk.chars() {
                    if ch == '\n' || ch == '\r' {
                        if !line_buffer.is_empty() {
                            progress.update_from_line(&line_buffer);
                            pb.set_message(progress.format());
                        }
                        line_buffer.clear();
                    } else {
                        line_buffer.push(ch);
                    }
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                _ => break,
            },
        }
    }

    pb.finish_and_clear();

    Ok(())
}

// --- Public API ---

pub fn sync_databases() -> Result<(), String> {
    run_pacman_sync()
}

pub fn upgrade_system(
    debug: bool,
    sync_first: bool,
    config: &crate::config::Config,
) -> Result<(), String> {
    if !util::is_root() {
        return Err("you cannot perform this operation unless you are root.".to_string());
    }

    if sync_first {
        run_pacman_sync()?;
    }
    let spinner = if debug {
        None
    } else {
        Some(util::create_spinner("Gathering stats"))
    };
    // After -Sy sync, databases are fresh so no need for temp sync
    let stats = get_stats(
        &config.display.parsed_stats(),
        debug,
        false,
        config,
        spinner.as_ref(),
    );
    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    if debug {
        crate::ui::display_stats(&stats, config);
        println!();
    } else if let Err(e) = crate::ui::display_stats_with_graphics(&stats, config) {
        eprintln!("error: {}", e);
        crate::ui::display_stats(&stats, config);
        println!();
    }

    run_pacman_pty(&["-Su"], true)
}

pub fn get_stats(
    requested: &[StatIdOrTitle],
    debug: bool,
    fresh_sync: bool,
    config: &crate::config::Config,
    spinner: Option<&ProgressBar>,
) -> PacmanStats {
    let ttl_minutes = config.cache.ttl_minutes;

    let total_start = Instant::now();
    let mut stats = PacmanStats::default();

    if needs_upgrade_stats(requested) {
        let start = Instant::now();
        let upgrade_stats = if fresh_sync {
            if debug {
                eprintln!("Using cached database (TTL {}min)", ttl_minutes);
            }
            calculate_upgrade_stats_with_sync(spinner, debug, ttl_minutes)
        } else {
            calculate_upgrade_stats("/var/lib/pacman", debug)
        };
        stats.total_upgradable = upgrade_stats.package_count;
        stats.download_size_mb = upgrade_stats.download_size_mb;
        stats.total_installed_size_mb = upgrade_stats.installed_size_mb;
        stats.net_upgrade_size_mb = upgrade_stats.net_upgrade_size_mb;
        if debug {
            eprintln!("Upgrade sizes + count: {:?}", start.elapsed());
        }
    } else if debug {
        eprintln!("Upgrade sizes: SKIP");
    }

    if needs_orphan_stats(requested) {
        let start = Instant::now();
        let (orphaned_count, orphaned_size) = get_orphaned_packages(debug);
        stats.orphaned_packages = orphaned_count;
        stats.orphaned_size_mb = orphaned_size;
        if debug {
            eprintln!("Orphaned packages: {:?}", start.elapsed());
        }
    } else if debug {
        eprintln!("Orphaned packages: SKIP");
    }

    let sync_handle = if needs_mirror_url(requested) {
        let start = Instant::now();
        stats.mirror_url = get_mirror_url();
        if debug {
            eprintln!("Mirror URL: {:?}", start.elapsed());
        }

        if needs_mirror_health(requested) {
            let sync_start = Instant::now();
            let mirror_url_clone = stats.mirror_url.clone();
            let handle = std::thread::spawn(move || {
                mirror_url_clone
                    .as_ref()
                    .and_then(|url| check_mirror_sync(url, debug))
            });
            Some((handle, sync_start))
        } else {
            if debug {
                eprintln!("Mirror sync age: SKIP");
            }
            None
        }
    } else {
        if debug {
            eprintln!("Mirror URL: SKIP");
            eprintln!("Mirror sync age: SKIP");
        }
        None
    };

    if requested.iter().any(|s| matches!(s, StatIdOrTitle::Stat(StatId::Installed))) {
        let start = Instant::now();
        stats.total_installed = get_installed_count();
        if debug {
            eprintln!("Installed count: {:?}", start.elapsed());
        }
    }

    if requested.iter().any(|s| matches!(s, StatIdOrTitle::Stat(StatId::LastUpdate))) {
        let start = Instant::now();
        stats.days_since_last_update = get_seconds_since_update();
        if debug {
            eprintln!("Last update time: {:?}", start.elapsed());
        }
    }

    if requested.iter().any(|s| matches!(s, StatIdOrTitle::Stat(StatId::CacheSize))) {
        let start = Instant::now();
        stats.cache_size_mb = get_cache_size();
        if debug {
            eprintln!("Cache size: {:?}", start.elapsed());
        }
    }

    if needs_disk_stat(requested) {
        let start = Instant::now();
        if let Some((used, total)) = get_disk_usage(&config.disk.path) {
            stats.disk_used_bytes = Some(used);
            stats.disk_total_bytes = Some(total);
        }
        if debug {
            eprintln!("Disk usage: {:?}", start.elapsed());
        }
    } else if debug {
        eprintln!("Disk usage: SKIP");
    }

    let start = Instant::now();
    stats.pacman_version = get_pacman_version();
    if debug {
        eprintln!("Pacman version: {:?}", start.elapsed());
    }

    if let Some((handle, sync_start)) = sync_handle {
        if let Some(pb) = spinner {
            pb.set_message("Checking mirror last sync");
        }
        stats.mirror_sync_age_hours = handle.join().ok().flatten();
        if debug {
            eprintln!("Mirror sync age: {:?}", sync_start.elapsed());
        }
    }

    if debug {
        eprintln!("TOTAL: {:?}\n", total_start.elapsed());
    }

    stats
}
