use std::fs;
use std::path::{Path, PathBuf};
use crate::config::{self, Config};
use crate::{log, path, tex, unpacker, wallpaper};

pub fn print_help() {
    println!("lianpkg — Steam wallpaper extract & convert tool");
    println!();
    println!("Usage:");
    println!("  lianpkg <mode> [args]");
    println!();
    println!("Modes:");
    println!("  wallpaper    Extract wallpapers");
    println!("  pkg          Unpack .pkg files");
    println!("  tex          Convert .tex to images");
    println!("  auto         Run all steps");
    println!();
    println!("Options:");
    println!("  -h, --help   Show this help");
    println!("  -d, --debug  Enable debug log");
    println!();
    println!("Please read the README for full documentation ❤");
}


pub fn run_wallpaper(config: &Config, args: &[String]) -> wallpaper::WallpaperStats {
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

    wallpaper::extract_wallpapers(&search_path, &output_path, video_path.as_deref())
}

pub fn run_pkg(config: &Config, args: &[String]) -> usize {
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
        return 0;
    }

    let files = path::get_target_files(&input_path);
    let pkg_files: Vec<_> = files.into_iter().filter(|f| f.extension().map_or(false, |e| e == "pkg")).collect();

    if pkg_files.is_empty() {
        log::info("No .pkg files found.");
        return 0;
    }

    let mut count = 0;
    for file in pkg_files {
        let file_stem = file.file_stem().unwrap().to_str().unwrap();
        let pkg_output_dir = get_unique_output_path(&output_path, file_stem);
        
        if let Err(e) = fs::create_dir_all(&pkg_output_dir) {
            log::error(&format!("Failed to create output dir: {}", e));
            continue;
        }
        unpacker::unpack_pkg(&file, &pkg_output_dir);
        count += 1;
    }
    count
}

pub fn run_tex(config: &Config, args: &[String]) -> usize {
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
        return 0;
    }

    let files = path::get_target_files(&input_path);
    let tex_files: Vec<_> = files.into_iter().filter(|f| f.extension().map_or(false, |e| e == "tex")).collect();

    if tex_files.is_empty() {
        log::info("No .tex files found.");
        return 0;
    }

    let mut count = 0;
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
        count += 1;
    }
    count
}

pub fn run_auto(config: &Config) {
    log::title("🤖 Starting Auto Mode");
    let wp_stats = run_wallpaper(config, &[]);
    let pkg_count = run_pkg(config, &[]);
    let tex_count = run_tex(config, &[]);
    
    log::title("✨ Auto Mode Completed ✨");
    println!("==========================================");
    println!("             Summary Report               ");
    println!("==========================================");
    println!("Wallpaper Extraction:");
    println!("  - Videos Extracted: {}", wp_stats.mp4_count);
    println!("  - PKGs Extracted:   {}", wp_stats.pkg_count);
    println!("PKG Unpacking:");
    println!("  - PKGs Unpacked:    {}", pkg_count);
    println!("TEX Conversion:");
    println!("  - TEXs Converted:   {}", tex_count);
    println!("==========================================");
}

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
