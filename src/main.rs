mod core;
mod managers;
mod ui;

use std::sync::mpsc;
use std::thread;

fn main() {
    //checking for flags
    let args: Vec<String> = std::env::args().collect();
    let text_mode= args.contains(&"--text".to_string()) || args.contains(&"-t".to_string());

    println!();

    // Get all local stats + fast network operations (mirror URL, sync age)
    let stats = core::get_manager_stats();

    if text_mode {
        let mirror = core::test_mirror_health();
        ui::display_stats(&stats);
        ui::display_mirror_health(&mirror, &stats);
    } else {
        if let Some(ref mirror_url) = stats.mirror_url {
            // Have mirror URL - spawn thread for speed test
            let mirror_url = mirror_url.clone();
            let (progress_tx, progress_rx) = mpsc::channel();
            let (speed_tx, speed_rx) = mpsc::channel();

            thread::spawn(move || {
                let speed = core::test_mirror_speed_with_progress(&mirror_url, |progress| {
                    let _ = progress_tx.send(progress);
                });
                let _ = speed_tx.send(speed);
            });

            if let Err(e) = ui::display_stats_with_graphics(&stats, progress_rx, speed_rx) {
                eprintln!("Error running TUI: {}", e);
            }
        } else {
            ui::display_stats(&stats);
        }
    }
}
