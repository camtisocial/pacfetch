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

#[derive(Debug, Clone, PartialEq)]
pub enum StatIdOrTitle {
    Stat(StatId),
    NamedTitle(String),
    LegacyTitle,
}

const BYTES_PER_GIB: f64 = 1073741824.0;

impl StatId {
    /// Parse a stat string, handling both regular stats and title.{name} references
    pub fn parse(s: &str) -> Result<StatIdOrTitle, String> {
        if let Some(name) = s.strip_prefix("title.") {
            if name.is_empty() {
                return Err("title name cannot be empty".to_string());
            }
            return Ok(StatIdOrTitle::NamedTitle(name.to_string()));
        }

        if s == "title" {
            return Ok(StatIdOrTitle::LegacyTitle);
        }

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

    pub fn config_key(&self) -> &'static str {
        match self {
            StatId::Title => "title",
            StatId::Installed => "installed",
            StatId::Upgradable => "upgradable",
            StatId::LastUpdate => "last_update",
            StatId::DownloadSize => "download_size",
            StatId::InstalledSize => "installed_size",
            StatId::NetUpgradeSize => "net_upgrade_size",
            StatId::OrphanedPackages => "orphaned_packages",
            StatId::CacheSize => "cache_size",
            StatId::MirrorUrl => "mirror_url",
            StatId::MirrorHealth => "mirror_health",
            StatId::Disk => "disk",
        }
    }

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
    requested
        .iter()
        .any(|s| matches!(s, StatIdOrTitle::Stat(StatId::OrphanedPackages)))
}

pub fn needs_mirror_health(requested: &[StatIdOrTitle]) -> bool {
    requested
        .iter()
        .any(|s| matches!(s, StatIdOrTitle::Stat(StatId::MirrorHealth)))
}

pub fn needs_mirror_url(requested: &[StatIdOrTitle]) -> bool {
    requested.iter().any(|s| {
        matches!(s, StatIdOrTitle::Stat(StatId::MirrorUrl))
            || matches!(s, StatIdOrTitle::Stat(StatId::MirrorHealth))
    })
}

pub fn needs_disk_stat(requested: &[StatIdOrTitle]) -> bool {
    requested
        .iter()
        .any(|s| matches!(s, StatIdOrTitle::Stat(StatId::Disk)))
}
