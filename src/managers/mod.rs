
pub struct ManagerStats {
    pub total_installed: u32,
    pub total_upgradable: u32,
    pub days_since_last_update: u32,
}

pub trait PackageManager {
    fn get_stats(&self) -> ManagerStats;
}

pub struct DummyManager;

impl PackageManager for DummyManager{
    fn get_stats(&self) -> ManagerStats{
        ManagerStats {
            total_installed: 123,
            total_upgradable: 45,
            days_since_last_update: 7,
        }
    }
}
