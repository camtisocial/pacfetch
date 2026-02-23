mod color;
mod config;
mod log;
mod pacman;
mod stats;
mod ui;
mod util;

use clap::{CommandFactory, Parser};
use config::Config;
use std::fs;

fn ensure_config_exists() {
    let Some(config_path) = Config::config_path() else {
        return;
    };

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&config_path, include_str!("../default_config.toml"));
    }
}

#[derive(Parser)]
#[command(name = "pacfetch")]
#[command(version)]
#[command(about = "A neofetch style wrapper for pacman's Syu/Sy/Su commands")]
#[command(after_help = "\
Commands:
   no args      Show stats with sync to temp databases
  -Sy           Sync package databases
  -Su           Upgrade system
  -Syu          Sync databases and upgrade system
  --yay         Full system + AUR upgrade via yay
  --paru        Full system + AUR upgrade via paru

Options:
      --ascii <ASCII>  Use custom ASCII art (path, built-in name, or NONE)
      --color <COLOR>  Override ASCII art color (name, hex #RRGGBB, or none)
      --image <PATH>   Use an image instead of ASCII art
      --json           Output stats as JSON
      --local          Use local cached database (skip temp sync)
  -d, --debug          Debug mode
  -h, --help           Print help
  -V, --version        Print version")]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
struct Cli {
    #[arg(short = 'S', hide = true)]
    sync_op: bool,

    #[arg(short = 'y', hide = true)]
    sync_db: bool,

    #[arg(short = 'u', hide = true)]
    upgrade: bool,

    #[arg(short, long, hide = true)]
    debug: bool,

    #[arg(long, hide = true)]
    local: bool,

    #[arg(short = 'h', long = "help", hide = true)]
    help: bool,

    #[arg(short = 'V', short_alias = 'v', long = "version", hide = true)]
    version: bool,

    #[arg(long = "ascii", hide = true)]
    ascii: Option<String>,

    #[arg(long = "color", hide = true)]
    color: Option<String>,

    #[arg(long = "image", hide = true)]
    image: Option<String>,

    #[arg(long = "json", hide = true)]
    json: bool,

    #[arg(long = "yay", hide = true)]
    yay: bool,

    #[arg(long = "paru", hide = true)]
    paru: bool,
}

fn is_bare_invocation(cli: &Cli) -> bool {
    !cli.sync_op && !cli.sync_db && !cli.upgrade && !cli.yay && !cli.paru && !cli.local
}

fn print_error_and_help(msg: &str) -> ! {
    eprintln!("error: {}\n", msg);
    let _ = Cli::command().print_help();
    eprintln!();
    std::process::exit(1);
}

fn main() {
    ensure_config_exists();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(_) => print_error_and_help("unrecognized flag"),
    };

    if cli.help {
        let _ = Cli::command().print_help();
        println!();
        std::process::exit(0);
    }

    if cli.version {
        println!("pacfetch {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    // Load config
    let mut config = Config::load();

    if let Some(ref ascii) = cli.ascii {
        config.display.ascii = ascii.clone();
    }

    if let Some(ref color) = cli.color {
        config.display.ascii_color = color.clone();
    }

    if let Some(ref image) = cli.image {
        config.display.image = image.clone();
    }

    let cli = if is_bare_invocation(&cli) && !config.default_args.is_empty() {
        let mut args = vec!["pacfetch".to_string()];
        args.extend(config.default_args.split_whitespace().map(String::from));
        if cli.debug {
            args.push("-d".to_string());
        }
        if let Some(ref ascii) = cli.ascii {
            args.push("--ascii".to_string());
            args.push(ascii.clone());
        }
        if let Some(ref color) = cli.color {
            args.push("--color".to_string());
            args.push(color.clone());
        }
        if let Some(ref image) = cli.image {
            args.push("--image".to_string());
            args.push(image.clone());
        }
        if cli.json {
            args.push("--json".to_string());
        }
        match Cli::try_parse_from(&args) {
            Ok(cli) => cli,
            Err(_) => {
                eprintln!(
                    "warning: invalid default_args in config: {:?}",
                    config.default_args
                );
                cli
            }
        }
    } else {
        cli
    };

    let invalid_flag = (cli.sync_op && !cli.sync_db && !cli.upgrade)
        || ((cli.sync_db || cli.upgrade) && !cli.sync_op);
    if invalid_flag {
        print_error_and_help("unrecognized flag combination");
    }

    // Handle system upgrade (-Su or -Syu)
    if cli.sync_op && cli.upgrade {
        let sync_first = cli.sync_db;
        if let Err(e) = pacman::upgrade_system(cli.debug, sync_first, &config) {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    // Handle --yay (full system + AUR upgrade via yay)
    if cli.yay {
        if let Err(e) = pacman::yay_upgrade(cli.debug, &config) {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    // Handle --paru (full system + AUR upgrade via paru)
    if cli.paru {
        if let Err(e) = pacman::paru_upgrade(cli.debug, &config) {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    // Skip fresh sync if: --local flag, or after -Sy
    let fresh_sync = !(cli.local || cli.sync_op && cli.sync_db);

    // Get stats
    let stats = if cli.sync_op && cli.sync_db {
        if let Err(e) = pacman::sync_databases() {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        let spinner = util::create_spinner("Gathering stats");
        let stats = pacman::get_stats(
            &config.display.parsed_stats(),
            cli.debug,
            fresh_sync,
            &config,
            Some(&spinner),
        );
        spinner.finish_and_clear();
        stats
    } else if cli.json {
        pacman::get_stats(
            &config.display.parsed_stats(),
            false,
            fresh_sync,
            &config,
            None,
        )
    } else if cli.debug {
        println!();
        pacman::get_stats(
            &config.display.parsed_stats(),
            cli.debug,
            fresh_sync,
            &config,
            None,
        )
    } else {
        let spinner = util::create_spinner("Gathering stats");
        let stats = pacman::get_stats(
            &config.display.parsed_stats(),
            cli.debug,
            fresh_sync,
            &config,
            Some(&spinner),
        );
        spinner.finish_and_clear();
        stats
    };

    if cli.json {
        println!("{}", stats_to_json_string(&stats));
    } else if cli.debug {
        ui::display_stats(&stats, &config);
        println!();
    } else if let Err(e) = ui::display_stats_with_graphics(&stats, &config) {
        eprintln!("error: {}", e);
    }
}

fn stats_to_json_string(stats: &pacman::PacmanStats) -> String {
    let mut map = serde_json::Map::new();
    for id in stats::ALL_STAT_IDS {
        if let Some(value) = id.format_value(stats) {
            map.insert(
                id.config_key().to_string(),
                serde_json::Value::String(value),
            );
        }
    }
    serde_json::to_string_pretty(&serde_json::Value::Object(map))
        .unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pacman::PacmanStats;

    #[test]
    fn test_json_contains_installed_and_upgradable() {
        let stats = PacmanStats {
            total_installed: 1234,
            total_upgradable: 5,
            ..Default::default()
        };
        let output = stats_to_json_string(&stats);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["installed"], "1234");
        assert_eq!(parsed["upgradable"], "5");
    }

    #[test]
    fn test_json_omits_none_values() {
        let stats = PacmanStats {
            total_installed: 100,
            total_upgradable: 0,
            ..Default::default()
        };
        let output = stats_to_json_string(&stats);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("download_size").is_none());
    }
}
