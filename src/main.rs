mod config;
mod log;
mod path;
mod tex;
mod unpacker;
mod wallpaper;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use config::{Config, ConfigStatus};

fn get_unique_output_path(base: &Path, name: &str) -> PathBuf {
    let mut target = base.join(name);
    if !target.exists() {
        return target;
    }

    let mut i = 1;
    loop {
        let new_name = format!("{}-{}", name, i);
        target = base.join(&new_name);
        if !target.exists() {
            return target;
        }
        i += 1;
    }
}

fn print_help() {
    println!("==========================================");
    println!("                  LianPkg                 ");
    println!("==========================================");
    println!("Usage: lianpkg [MODE] [OPTIONS]");
    println!();
    println!("Modes:");
    println!("  wallpaper [SEARCH_PATH] [OUTPUT_PATH]");
    println!("      Extract wallpapers from Steam Workshop.");
    println!("      SEARCH_PATH: Path to search for wallpapers (Optional, overrides config)");
    println!("      OUTPUT_PATH: Path to output extracted files (Optional, overrides config)");
    println!();
    println!("  pkg [INPUT_PATH] [OUTPUT_PATH]");
    println!("      Unpack .pkg files.");
    println!("      INPUT_PATH: Directory containing .pkg files (Optional, defaults to wallpaper output/Pkg)");
    println!("      OUTPUT_PATH: Directory to output unpacked files (Optional, overrides config)");
    println!();
    println!("  tex [INPUT_PATH]");
    println!("      Convert .tex files to images.");
    println!("      INPUT_PATH: Directory containing unpacked files (Optional, defaults to pkg output)");
    println!();
    println!("  auto");
    println!("      Run all steps in sequence: wallpaper -> pkg -> tex.");
    println!("      Uses configuration values for paths.");
    println!();
    println!("Options:");
    println!("  -h, --help  Print this help message");
    println!("  -d, --debug Enable debug logging");
    println!();
    println!("Configuration:");
    println!("  Config file is located at ~/.config/lianpkg/config.toml");
    println!("  (or default.toml if config.toml does not exist)");
    println!("  Note: If both exist, the program will exit with an error.");
}


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
        print_help();
        return;
    }

    let mode = &args[1];
    
    if mode == "-h" || mode == "--help" {
        print_help();
        return;
    }

    let debug_mode = args.iter().any(|a| a == "-d" || a == "--debug");
    log::set_debug(debug_mode);

    match mode.as_str() {
        "wallpaper" => run_wallpaper(&config, &args[2..]),
        "pkg" => run_pkg(&config, &args[2..]),
        "tex" => run_tex(&config, &args[2..]),
        "auto" => run_auto(&config),
        _ => {
            println!("Unknown mode: {}", mode);
            print_help();
        }
    }
}

fn run_wallpaper(config: &Config, args: &[String]) {
    let search_path = if args.len() > 0 && !args[0].starts_with("-") {
        config::expand_path(&args[0])
    } else {
        config::expand_path(&config.wallpaper.search_path)
    };

    let output_path = if args.len() > 1 && !args[1].starts_with("-") {
        config::expand_path(&args[1])
    } else {
        config::expand_path(&config.wallpaper.output_path)
    };
    
    let video_path = config.wallpaper.video_path.as_ref().map(|s| config::expand_path(s));

    wallpaper::extract_wallpapers(&search_path, &output_path, video_path.as_deref());
}

fn run_pkg(config: &Config, args: &[String]) {
    let default_input = config::expand_path(&config.wallpaper.output_path).join("Pkg");
    let default_output = config::expand_path(&config.pkg.output_path);

    let input_path = if args.len() > 0 && !args[0].starts_with("-") {
        config::expand_path(&args[0])
    } else {
        default_input
    };

    let output_path = if args.len() > 1 && !args[1].starts_with("-") {
        config::expand_path(&args[1])
    } else {
        default_output
    };

    log::title("🚀 Starting PKG Unpack");
    log::info(&format!("Input: {:?}", input_path));
    log::info(&format!("Output: {:?}", output_path));

    if !input_path.exists() {
        log::error("Input path does not exist");
        return;
    }

    let files = path::get_target_files(&input_path);
    let pkg_files: Vec<_> = files.into_iter().filter(|f| f.extension().map_or(false, |e| e == "pkg")).collect();

    if pkg_files.is_empty() {
        log::info("No .pkg files found.");
        return;
    }

    for file in pkg_files {
        let file_stem = file.file_stem().unwrap().to_str().unwrap();
        let pkg_output_dir = get_unique_output_path(&output_path, file_stem);
        
        if let Err(e) = fs::create_dir_all(&pkg_output_dir) {
            log::error(&format!("Failed to create output dir: {}", e));
            continue;
        }
        unpacker::unpack_pkg(&file, &pkg_output_dir);
    }
}

fn run_tex(config: &Config, args: &[String]) {
    let default_input = config::expand_path(&config.pkg.output_path);
    
    let input_path = if args.len() > 0 && !args[0].starts_with("-") {
        config::expand_path(&args[0])
    } else {
        default_input
    };

    log::title("🚀 Starting TEX Conversion");
    log::info(&format!("Input: {:?}", input_path));

    if !input_path.exists() {
        log::error("Input path does not exist");
        return;
    }

    let files = path::get_target_files(&input_path);
    let tex_files: Vec<_> = files.into_iter().filter(|f| f.extension().map_or(false, |e| e == "tex")).collect();

    if tex_files.is_empty() {
        log::info("No .tex files found.");
        return;
    }

    for file in tex_files {
        let file_stem = file.file_stem().unwrap().to_str().unwrap();
        let project_root = path::find_project_root(&file);
        
        let (base_output_dir, relative_path) = if let Some(root) = project_root {
             let relative = file.strip_prefix(&root).unwrap_or(Path::new(file_stem));
             (root.join("tex_converted"), relative.parent().unwrap_or(Path::new("")).to_path_buf())
        } else {
             (file.parent().unwrap().join("tex_converted"), PathBuf::new())
        };

        let final_output_dir = base_output_dir.join(relative_path);
        if let Err(e) = fs::create_dir_all(&final_output_dir) {
             log::error(&format!("Failed to create output dir: {}", e));
             continue;
        }
        let output_filename = final_output_dir.join(file_stem);
        tex::process_tex(&file, &output_filename);
    }
}

fn run_auto(config: &Config) {
    log::title("🤖 Starting Auto Mode");
    run_wallpaper(config, &[]);
    run_pkg(config, &[]);
    run_tex(config, &[]);
    log::title("✨ Auto Mode Completed ✨");
}

