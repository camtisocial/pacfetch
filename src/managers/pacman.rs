use crate::managers::{ManagerStats, PackageManager};
use alpm::Alpm;
use chrono::{DateTime, FixedOffset, Local};
use std::fs;
use std::process::Command;

pub struct FetchPacmanStats;

impl FetchPacmanStats {
    /// Get the count of installed packages using pacman -Q
    fn get_installed_count(&self) -> u32 {
        let output = Command::new("pacman").arg("-Q").output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines().count() as u32
    }

    /// Get the count of upgradable packages using checkupdates
    fn get_upgradable_count(&self) -> u32 {
        let output = Command::new("pacman").arg("-Qu").output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.lines().count() as u32
    }

    /// get time since last update from /var/log/pacman.log
    /// returns seconds
    fn get_seconds_since_update(&self) -> Option<i64> {
        /*
        This is checking the log for the last time the user ran pacman -Syu, and packages
        actually installed thereafter, that way we are actually returning last time updated
        rathern than last time -Syu was run, ideally. kind of janky
        */

        let contents =
            fs::read_to_string("/var/log/pacman.log").expect("Failed to read pacman.log");

        let mut saw_syu = false;
        let mut saw_sync = false;
        let mut saw_upgrade_start = false;
        let mut saw_alpm = false;
        let mut block_timestamp: Option<String> = None;
        let mut last_valid_timestamp: Option<String> = None;

        for line in contents.lines() {
            let trimmed = line.trim();

            // Extract timestamp of any line
            // Format: [2025-12-05T15:43:51-0800] ...
            let timestamp = trimmed
                .split(']')
                .next()
                .map(|x| x.trim_start_matches('['))
                .unwrap_or("");

            // look for start of syu block, then alpm lines to make sure the update actually
            // started
            if trimmed.contains("Running 'pacman -Syu'") {
                // reset tracking
                saw_syu = true;
                saw_sync = false;
                saw_upgrade_start = false;
                saw_alpm = false;
                block_timestamp = Some(timestamp.to_string());
                continue;
            }

            if saw_syu && trimmed.contains("synchronizing package lists") {
                saw_sync = true;
                continue;
            }

            if saw_sync && trimmed.contains("starting full system upgrade") {
                saw_upgrade_start = true;
                continue;
            }

            if saw_upgrade_start && trimmed.contains("[ALPM]") {
                saw_alpm = true;
            }

            // If we start another pacman run before ALPM, the previous wasn't real
            if trimmed.contains("[PACMAN] Running") && !trimmed.contains("pacman -Syu") {
                // restart
                saw_syu = false;
                saw_sync = false;
                saw_upgrade_start = false;
                saw_alpm = false;
                block_timestamp = None;
                continue;
            }

            // if the block is complete, update last_valid_timestamp
            if saw_syu && saw_sync && saw_upgrade_start && saw_alpm {
                last_valid_timestamp = block_timestamp.clone();
            }
        }

        if let Some(ts) = last_valid_timestamp {
            // confert to RFC3339
            let formatted_date = format!("{}:{}", &ts[..22], &ts[22..]);

            let parsed: DateTime<FixedOffset> =
                DateTime::parse_from_rfc3339(&formatted_date).unwrap();

            let last_update_local = parsed.with_timezone(&Local);
            let now = Local::now();
            let duration = now.signed_duration_since(last_update_local);
            let seconds = duration.num_seconds().max(0);

            return Some(seconds);
        }

        None
    }

    fn get_upgrade_sizes(&self) -> (Option<f64>, Option<f64>, Option<f64>) {

        // ########## Creating alpm connection, getting sync db and local db #########

        let mut alpm = match Alpm::new("/", "/var/lib/pacman") {
            Ok(a) => a,
            Err(_) => return (None, None, None),
        };

        // Register sync databases
        let _ = alpm.register_syncdb_mut("core", alpm::SigLevel::NONE);
        let _ = alpm.register_syncdb_mut("extra", alpm::SigLevel::NONE);
        let _ = alpm.register_syncdb_mut("multilib", alpm::SigLevel::NONE);

        // Set NO_LOCK, avoids needing root
        if alpm.trans_init(alpm::TransFlag::NO_LOCK).is_err() {
            return (None, None, None);
        }

        // Add sysupgrade to transaction
        if alpm.sync_sysupgrade(false).is_err() {
            let _ = alpm.trans_release();
            return (None, None, None);
        }

        // Prepare the transaction
        if alpm.trans_prepare().is_err() {
            let _ = alpm.trans_release();
            return (None, None, None);
        }

        // Get local database for comparing old vs new sizes
        let localdb = alpm.localdb();

        let mut total_download_size: i64 = 0;
        let mut total_installed_size: i64 = 0;
        let mut net_upgrade_size: i64 = 0;


        // ########## comparing values/accumulating totals #########

        // Get packages to be upgraded and calculate sizes
        for pkg in alpm.trans_add().into_iter() {
            total_download_size += pkg.download_size();
            let new_size = pkg.isize();
            total_installed_size += new_size;

            // Check if this is an upgrade
            if let Ok(oldpkg) = localdb.pkg(pkg.name()) {
                // upgrade
                let old_size = oldpkg.isize();
                net_upgrade_size += new_size - old_size;
            } else {
                // new install: add full size
                net_upgrade_size += new_size;
            }
        }


        // ############ Cleaning up handles/transaction and data ##########

        // Handle removals
        for pkg in alpm.trans_remove().into_iter() {
            net_upgrade_size -= pkg.isize();
        }

        // Release transaction
        let _ = alpm.trans_release();

        // Convert to MiB
        let download_mib = total_download_size as f64 / 1048576.0;
        let installed_mib = total_installed_size as f64 / 1048576.0;
        let mut net_mib = net_upgrade_size as f64 / 1048576.0;

        // Avoid -0.00 display issue
        if net_mib > -0.01 && net_mib < 0.01 {
            net_mib = 0.0;
        }

        (Some(download_mib), Some(installed_mib), Some(net_mib))
    }


    fn get_orphaned_packages(&self) -> (Option<u32>, Option<f64>) {
        let alpm = match Alpm::new("/", "/var/lib/pacman") {
            Ok(a) => a,
            Err(_) => return (None, None),
        };

        let localdb = alpm.localdb();
        let mut count = 0;
        let mut total_size: i64 = 0;

        // Find packages installed as dependencies that nothing depends on
        for pkg in localdb.pkgs().into_iter() {
            // Check if installed as a dependency (not explicitly installed)
            if pkg.reason() == alpm::PackageReason::Depend {
                // Check if anything requires this package
                if pkg.required_by().len() == 0 && pkg.optional_for().len() == 0 {
                    count += 1;
                    total_size += pkg.isize();
                }
            }
        }

        let size_mb = total_size as f64 / 1048576.0;
        (Some(count), Some(size_mb))
    }

    fn get_cache_size(&self) -> Option<f64> {
        let cache_path = std::path::Path::new("/var/cache/pacman/pkg");

        if let Ok(entries) = std::fs::read_dir(cache_path) {
            let total_size: u64 = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.metadata().ok())
                .filter(|m| m.is_file())
                .map(|m| m.len())
                .sum();

            Some(total_size as f64 / 1048576.0)
        } else {
            None
        }
    }

    fn get_mirror_health(&self) -> Option<String> {
        Some("test".to_string())
    }
}

impl PackageManager for FetchPacmanStats {
    fn get_stats(&self) -> ManagerStats {
        let (download_size, total_installed_size, net_upgrade_size) = self.get_upgrade_sizes();
        let (orphaned_count, orphaned_size) = self.get_orphaned_packages();

        ManagerStats {
            total_installed: self.get_installed_count(),
            total_upgradable: self.get_upgradable_count(),
            days_since_last_update: self.get_seconds_since_update(),
            mirror_health: self.get_mirror_health(),
            download_size_mb: download_size,
            total_installed_size_mb: total_installed_size,
            net_upgrade_size_mb: net_upgrade_size,
            orphaned_packages: orphaned_count,
            orphaned_size_mb: orphaned_size,
            cache_size_mb: self.get_cache_size(),
        }
    }
}
