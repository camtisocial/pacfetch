use crate::managers::{ManagerStats, PackageManager};

pub struct PacmanStats;

impl PackageManager for PacmanStats {
    fn get_stats(&self) -> ManagerStats {
        ManagerStats {
            total_installed: 200,
            total_upgradable: 30,
            days_since_last_update: 5,
        }
    }
}

