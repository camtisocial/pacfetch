use crate::core;


pub fn draw_ui() {
    let stats = core::get_manager_stats();

    println!("----- upkg -----");
    println!("Total Installed Packages: {}", stats.total_installed);
    println!("Total Upgradable Packages: {}", stats.total_upgradable);
    println!("Days Since Last Update: {}", stats.days_since_last_update);

}
