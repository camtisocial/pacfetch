use crate::managers::{DummyManager, ManagerStats, PackageManager};

pub fn get_manager_stats() -> ManagerStats {
    let backend = DummyManager;
    backend.get_stats()
}
