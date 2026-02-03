use serde::Deserialize;

use crate::pacman::PacmanStats;
use crate::util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatId {
    Title,
    Installed,
    Upgradable,
    LastUpdate,
    DownloadSize,
    InstalledSize,
    NetUpgradeSize,
    OrphanedPackages,
    CacheSize,
    MirrorUrl,
    MirrorHealth,
    Disk,
}

const BYTES_PER_GIB: f64 = 1073741824.0;

impl StatId {
    pub fn label(&self) -> &'static str {
        match self {
            StatId::Title => "",
            StatId::Installed => "Installed",
            StatId::Upgradable => "Upgradable",
            StatId::LastUpdate => "Last System Update",
            StatId::DownloadSize => "Download Size",
            StatId::InstalledSize => "Installed Size",
            StatId::NetUpgradeSize => "Net Upgrade Size",
            StatId::OrphanedPackages => "Orphaned Packages",
            StatId::CacheSize => "Package Cache",
            StatId::MirrorUrl => "Mirror URL",
            StatId::MirrorHealth => "Mirror Health",
            StatId::Disk => "Disk",
        }
    }

    pub fn format_value(&self, stats: &PacmanStats) -> Option<String> {
        match self {
            StatId::Title => None,
            StatId::Installed => Some(stats.total_installed.to_string()),
            StatId::Upgradable => Some(stats.total_upgradable.to_string()),
            StatId::LastUpdate => stats.days_since_last_update.map(util::normalize_duration),
            StatId::DownloadSize => stats.download_size_mb.map(|s| format!("{:.2} MiB", s)),
            StatId::InstalledSize => stats
                .total_installed_size_mb
                .map(|s| format!("{:.2} MiB", s)),
            StatId::NetUpgradeSize => stats.net_upgrade_size_mb.map(|s| format!("{:.2} MiB", s)),
            StatId::OrphanedPackages => {
                if let Some(count) = stats.orphaned_packages {
                    if count > 0 {
                        if let Some(size) = stats.orphaned_size_mb {
                            Some(format!("{} ({:.2} MiB)", count, size))
                        } else {
                            Some(count.to_string())
                        }
                    } else {
                        Some("0".to_string())
                    }
                } else {
                    None
                }
            }
            StatId::CacheSize => stats.cache_size_mb.map(|s| format!("{:.2} MiB", s)),
            StatId::MirrorUrl => stats.mirror_url.clone(),
            StatId::MirrorHealth => match (&stats.mirror_url, stats.mirror_sync_age_hours) {
                (Some(_), Some(age)) => Some(format!("OK (last sync {:.1} hours)", age)),
                (Some(_), None) => Some("Err - could not check sync status".to_string()),
                (None, _) => Some("Err - no mirror found".to_string()),
            },
            StatId::Disk => {
                if let (Some(used), Some(total)) = (stats.disk_used_bytes, stats.disk_total_bytes) {
                    let used_gib = used as f64 / BYTES_PER_GIB;
                    let total_gib = total as f64 / BYTES_PER_GIB;
                    let pct = if total > 0 {
                        (used as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    };
                    Some(format!(
                        "{:.2} GiB / {:.2} GiB ({:.0}%)",
                        used_gib, total_gib, pct
                    ))
                } else {
                    None
                }
            }
        }
    }
}

// --- stat fetch request helpers ---
pub fn needs_upgrade_stats(requested: &[StatId]) -> bool {
    requested.iter().any(|s| {
        matches!(
            s,
            StatId::Upgradable
                | StatId::DownloadSize
                | StatId::InstalledSize
                | StatId::NetUpgradeSize
        )
    })
}

pub fn needs_orphan_stats(requested: &[StatId]) -> bool {
    requested.contains(&StatId::OrphanedPackages)
}

pub fn needs_mirror_health(requested: &[StatId]) -> bool {
    requested.contains(&StatId::MirrorHealth)
}

pub fn needs_mirror_url(requested: &[StatId]) -> bool {
    requested.contains(&StatId::MirrorUrl) || needs_mirror_health(requested)
}

pub fn needs_disk_stat(requested: &[StatId]) -> bool {
    requested.contains(&StatId::Disk)
}
