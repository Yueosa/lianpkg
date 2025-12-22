mod commands;
mod config;
mod log;
mod path;
mod tex;
mod unpacker;
mod wallpaper;

use std::env;
use config::ConfigStatus;

fn main() {
    let config = match config::load_config() {
        ConfigStatus::Loaded(c) => c,
        ConfigStatus::CreatedDefault(path) => {
            println!("First run detected. Configuration file created at: {:?}", path);
            println!("Please edit the configuration file if needed and run the program again.");
            return;
        }
        ConfigStatus::Error(e) => {
            eprintln!("[ERROR] {}", e);
            return;
        }
    };

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        commands::print_help();
        return;
    }

    let mode = &args[1];
    
    if mode == "-h" || mode == "--help" {
        commands::print_help();
        return;
    }

    let debug_mode = args.iter().any(|a| a == "-d" || a == "--debug");
    log::set_debug(debug_mode);

    match mode.as_str() {
        "wallpaper" => { commands::run_wallpaper(&config, &args[2..]); },
        "pkg" => { commands::run_pkg(&config, &args[2..]); },
        "tex" => { commands::run_tex(&config, &args[2..]); },
        "auto" => commands::run_auto(&config),
        _ => {
            println!("Unknown mode: {}", mode);
            commands::print_help();
        }
    }
}


