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
        config::expand_path(&config.wallpaper.workshop_path)
    };

    let raw_output_path = config::expand_path(&config.wallpaper.raw_output_path);
    let pkg_temp_path = config::expand_path(&config.wallpaper.pkg_temp_path);

    wallpaper::extract_wallpapers(&search_path, &raw_output_path, &pkg_temp_path)
}

pub fn run_pkg(config: &Config, args: &[String]) -> usize {
    let default_input = config::expand_path(&config.wallpaper.pkg_temp_path);
    let default_output = config::expand_path(&config.unpack.unpacked_output_path);

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

        // After unpacking, copy non-pkg resources from workshop_path if available
        // We need to find the corresponding raw folder in workshop_path.
        // The file_stem usually looks like "123456_scene" or just "scene" if we flattened it.
        // But in wallpaper::extract_wallpapers we named it "{dir_name}_{file_name}".
        // So we can try to split by first underscore to get the ID/DirName.
        
        // Heuristic: Try to find a folder in workshop_path that matches the prefix of the pkg file
        let workshop_path = config::expand_path(&config.wallpaper.workshop_path);
        if workshop_path.exists() {
             if let Some((dir_name, _)) = file_stem.split_once('_') {
                 let raw_source = workshop_path.join(dir_name);
                 if raw_source.exists() && raw_source.is_dir() {
                     // Target directory for resources: pkg_output_dir/tex_converted/
                     // If tex.converted_output_path is set, we should respect it, but here we are in 'pkg' mode,
                     // and usually resources go with the converted textures.
                     // However, since 'tex' mode runs AFTER 'pkg' mode, we can just put them in a 'tex_converted' folder inside the unpacked dir for now,
                     // or wherever the user expects them.
                     // The user requested: "Pkg_Unpacked/xxx_scene/tex_converted/"
                     // But if converted_output_path is set, we should use that instead.
                     
                     let resource_dest = path::resolve_tex_output_dir(config, &pkg_output_dir, None, None);
                     
                     log::info(&format!("Copying additional resources from {:?} to {:?}", raw_source, resource_dest));
                     if let Err(e) = copy_non_pkg_files(&raw_source, &resource_dest) {
                         log::error(&format!("Failed to copy resources: {}", e));
                     }
                 }
             }
        }

        count += 1;
    }
    count
}

fn copy_non_pkg_files(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            copy_non_pkg_files(&path, &dst.join(entry.file_name()))?;
        } else {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("pkg") {
                    continue;
                }
            }
            fs::copy(&path, dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn run_tex(config: &Config, args: &[String]) -> usize {
    let default_input = config::expand_path(&config.unpack.unpacked_output_path);
    
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
        
        // Determine output directory using centralized logic
        // We use input_path as the relative base if project_root is not found, or project_root if found.
        let relative_base = project_root.as_deref().unwrap_or(&input_path);
        let root_for_default = project_root.as_deref().unwrap_or(file.parent().unwrap());

        let final_output_dir = path::resolve_tex_output_dir(config, root_for_default, Some(&file), Some(relative_base));

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
    println!("  - Raw Wallpapers:   {}", wp_stats.raw_count);
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
