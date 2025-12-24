use std::fs;
use std::path::{Path, PathBuf};
use crate::config::Config;
use crate::{log, path, tex, unpacker, wallpaper};

pub fn run_wallpaper(config: &Config) -> Result<wallpaper::WallpaperStats, String> {
    let search_path = path::expand_path(&config.wallpaper.workshop_path);
    let raw_output_path = path::expand_path(&config.wallpaper.raw_output_path);
    let pkg_temp_path = path::expand_path(&config.wallpaper.pkg_temp_path);

    wallpaper::extract_wallpapers(&search_path, &raw_output_path, &pkg_temp_path, config.wallpaper.enable_raw_output)
}

pub fn run_pkg(config: &Config) -> Result<usize, String> {
    let input_path = path::expand_path(&config.wallpaper.pkg_temp_path);
    let output_path = path::expand_path(&config.unpack.unpacked_output_path);

    log::title("🚀 Starting PKG Unpack");
    log::info(&format!("Input: {:?}", input_path));
    log::info(&format!("Output: {:?}", output_path));

    if !input_path.exists() {
        return Err("Input path does not exist".to_string());
    }

    let files = path::get_target_files(&input_path);
    let pkg_files: Vec<_> = files.into_iter().filter(|f| f.extension().map_or(false, |e| e == "pkg")).collect();

    if pkg_files.is_empty() {
        log::info("No .pkg files found.");
        return Ok(0);
    }

    let mut count = 0;
    for file in pkg_files {
        let file_stem = file.file_stem().unwrap().to_str().unwrap();
        let pkg_output_dir = get_unique_output_path(&output_path, file_stem);
        
        if let Err(e) = fs::create_dir_all(&pkg_output_dir) {
            return Err(format!("Failed to create output dir: {}", e));
        }
        unpacker::unpack_pkg(&file, &pkg_output_dir)?;

        let workshop_path = path::expand_path(&config.wallpaper.workshop_path);
        if workshop_path.exists() {
            let scene_name = path::scene_name_from_pkg_stem(file_stem);
            let raw_source = workshop_path.join(&scene_name);
            if raw_source.exists() && raw_source.is_dir() {
                let resource_dest = path::resolve_tex_output_dir(
                    config.tex.converted_output_path.as_deref(),
                    &pkg_output_dir,
                    None,
                    None,
                );
                
                log::info(&format!("Copying additional resources from {:?} to {:?}", raw_source, resource_dest));
                if let Err(e) = copy_non_pkg_files(&raw_source, &resource_dest) {
                    return Err(format!("Failed to copy resources: {}", e));
                }
            }
        }

        count += 1;
    }
    Ok(count)
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

pub fn run_tex(config: &Config) -> Result<usize, String> {
    let input_path = path::expand_path(&config.unpack.unpacked_output_path);

    log::title("🚀 Starting TEX Conversion");
    log::info(&format!("Input: {:?}", input_path));

    if !input_path.exists() {
        return Err("Input path does not exist".to_string());
    }

    let files = path::get_target_files(&input_path);
    let tex_files: Vec<_> = files.into_iter().filter(|f| f.extension().map_or(false, |e| e == "tex")).collect();

    if tex_files.is_empty() {
        log::info("No .tex files found.");
        return Ok(0);
    }

    let mut count = 0;
    for file in tex_files {
        let file_stem = file.file_stem().unwrap().to_str().unwrap();
        let project_root = path::find_project_root(&file);

        let scene_root = project_root.as_deref().unwrap_or_else(|| file.parent().unwrap());
        let relative_base = project_root.as_deref().unwrap_or(&input_path);

        let final_output_dir = path::resolve_tex_output_dir(
            config.tex.converted_output_path.as_deref(),
            scene_root,
            Some(&file),
            Some(relative_base),
        );

        if let Err(e) = fs::create_dir_all(&final_output_dir) {
            return Err(format!("Failed to create output dir: {}", e));
        }
        let output_filename = final_output_dir.join(file_stem);
        tex::process_tex(&file, &output_filename)?;
        count += 1;
    }
    Ok(count)
}

pub fn run_auto(config: &Config) {
    log::title("🤖 Starting Auto Mode");

    // Disk Usage Estimation
    let search_path = path::expand_path(&config.wallpaper.workshop_path);
    log::info("Calculating estimated disk usage...");
    let (pkg_size, raw_size) = wallpaper::estimate_requirements(&search_path, config.wallpaper.enable_raw_output);
    
    // Estimation logic:
    // Pkg_Temp: 1x PKG size
    // Pkg_Unpacked: ~1.5x PKG size (unpacked)
    // Tex_Converted: ~2.0x PKG size (textures)
    // Raw_Output: 1x Raw size
    let est_pkg_temp = pkg_size;
    let est_unpacked = (pkg_size as f64 * 1.5) as u64;
    let est_converted = (pkg_size as f64 * 2.0) as u64;
    let est_raw = raw_size;

    let peak_usage = est_pkg_temp + est_unpacked + est_converted + est_raw;
    let final_usage = est_raw + est_converted + 
        if config.unpack.clean_unpacked { 0 } else { est_unpacked } +
        if config.unpack.clean_pkg_temp { 0 } else { est_pkg_temp };

    use human_bytes::human_bytes;
    println!("==========================================");
    println!("          Disk Usage Estimation           ");
    println!("==========================================");
    println!("Found PKG Files:      {}", human_bytes(pkg_size as f64));
    if config.wallpaper.enable_raw_output {
        println!("Found Raw Files:      {}", human_bytes(raw_size as f64));
    } else {
        println!("Raw Files:            Skipped (--no-raw)");
    }
    println!("------------------------------------------");
    println!("Estimated Peak Usage: {}", human_bytes(peak_usage as f64));
    println!("Estimated Final Usage:{}", human_bytes(final_usage as f64));
    
    // Check available space
    let output_path = path::expand_path(&config.unpack.unpacked_output_path);
    // Try to find an existing parent to check space
    let mut check_path = output_path.clone();
    while !check_path.exists() {
        if let Some(parent) = check_path.parent() {
            check_path = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    if check_path.exists() {
        match fs2::available_space(&check_path) {
            Ok(available) => {
                println!("Available Space:      {}", human_bytes(available as f64));
                if available < peak_usage {
                    log::error("⚠️  WARNING: Insufficient disk space! ⚠️");
                    println!("You might run out of space during processing.");
                    println!("Required: {}, Available: {}", human_bytes(peak_usage as f64), human_bytes(available as f64));
                    println!("Press Ctrl+C to cancel or Enter to continue anyway...");
                    let _ = std::io::stdin().read_line(&mut String::new());
                } else {
                    println!("Disk Space Check:     OK ✅");
                }
            },
            Err(_) => {
                println!("Available Space:      Unknown (Check failed)");
            }
        }
    }
    println!("==========================================");
    
    let result = (|| -> Result<(wallpaper::WallpaperStats, usize, usize), String> {
        let wp_stats = run_wallpaper(config)?;
        let pkg_count = run_pkg(config)?;
        let tex_count = run_tex(config)?;
        Ok((wp_stats, pkg_count, tex_count))
    })();

    match result {
        Ok((wp_stats, pkg_count, tex_count)) => {
            if config.unpack.clean_pkg_temp {
                log::info("Cleaning Pkg_Temp...");
                if let Err(e) = cleanup_pkg_temp(&path::expand_path(&config.wallpaper.pkg_temp_path)) {
                    log::error(&format!("Cleanup Pkg_Temp failed: {}", e));
                }
            }

            if config.unpack.clean_unpacked {
                log::info("Cleaning Pkg_Unpacked (keeping tex_converted)...");
                if let Err(e) = cleanup_unpacked(&path::expand_path(&config.unpack.unpacked_output_path)) {
                    log::error(&format!("Cleanup Pkg_Unpacked failed: {}", e));
                }
            }
            
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
        },
        Err(e) => {
            log::error(&format!("❌ Auto Mode Failed: {}", e));
            log::info("Performing emergency cleanup...");
            
            // Always try to clean up temp files on error if they exist
            let _ = cleanup_pkg_temp(&path::expand_path(&config.wallpaper.pkg_temp_path));
            let _ = cleanup_unpacked(&path::expand_path(&config.unpack.unpacked_output_path));
            
            println!("==========================================");
            println!("             ERROR REPORT                 ");
            println!("==========================================");
            println!("An error occurred during execution:");
            println!("{}", e);
            println!("==========================================");
        }
    }
}

fn cleanup_pkg_temp(dir: &Path) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    fs::remove_dir_all(dir)
}

fn cleanup_unpacked(dir: &Path) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let scene_dir = entry.path();
        if !scene_dir.is_dir() {
            fs::remove_file(scene_dir)?;
            continue;
        }

        let tex_dir = scene_dir.join("tex_converted");
        if tex_dir.exists() {
            for child in fs::read_dir(&scene_dir)? {
                let child = child?;
                let child_path = child.path();
                if child_path == tex_dir {
                    continue;
                }
                if child_path.is_dir() {
                    fs::remove_dir_all(child_path)?;
                } else {
                    fs::remove_file(child_path)?;
                }
            }
        } else {
            fs::remove_dir_all(scene_dir)?;
        }
    }

    // Remove the directory itself if it is empty
    // 如果目录为空，则删除目录本身
    if let Ok(mut entries) = fs::read_dir(dir) {
        if entries.next().is_none() {
            fs::remove_dir(dir)?;
        }
    }

    Ok(())
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
