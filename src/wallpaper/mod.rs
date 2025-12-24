use std::fs;
use std::path::Path;
use crate::{log, path};

pub struct WallpaperStats {
    pub raw_count: usize,
    pub pkg_count: usize,
}

pub fn extract_wallpapers(search_path: &Path, raw_output: &Path, pkg_temp_output: &Path) -> Result<WallpaperStats, String> {
    let mut stats = WallpaperStats { raw_count: 0, pkg_count: 0 };
    
    if let Err(e) = fs::create_dir_all(raw_output) {
        return Err(format!("Failed to create raw output dir: {}", e));
    }
    if let Err(e) = fs::create_dir_all(pkg_temp_output) {
        return Err(format!("Failed to create pkg temp dir: {}", e));
    }

    log::title("Starting Wallpaper Extraction");
    log::debug("extract_wallpapers", &format!("Search: {:?}, Raw: {:?}, Pkg: {:?}", search_path, raw_output, pkg_temp_output), "Init");

    let entries = match fs::read_dir(search_path) {
        Ok(e) => e,
        Err(e) => {
            return Err(format!("Failed to read search path: {}", e));
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        let has_pkg = check_has_pkg(&path);

        if has_pkg {
            log::info(&format!("Found PKG in: {}", dir_name));
            
            if let Ok(sub_entries) = fs::read_dir(&path) {
                for sub_entry in sub_entries.flatten() {
                    let sub_path = sub_entry.path();
                    if let Some(ext) = sub_path.extension().and_then(|s| s.to_str()) {
                        if ext.eq_ignore_ascii_case("pkg") {
                            let file_name = sub_path.file_name().unwrap().to_str().unwrap();
                            let new_name = path::pkg_temp_dest(dir_name, file_name);
                            let dest = pkg_temp_output.join(new_name);
                            
                            log::debug("extract_wallpapers", &format!("Copying {:?}", sub_path), "Processing PKG");
                            if let Err(e) = fs::copy(&sub_path, &dest) {
                                return Err(format!("Failed to copy pkg: {}", e));
                            } else {
                                stats.pkg_count += 1;
                            }
                        }
                    }
                }
            }

        } else {
            log::info(&format!("Found Raw Wallpaper: {}", dir_name));
            let dest_dir = raw_output.join(dir_name);
            if dest_dir.exists() {
                log::debug("extract_wallpapers", dir_name, "Skipping existing raw wallpaper");
                continue;
            }

            if let Err(e) = copy_dir_recursive(&path, &dest_dir) {
                return Err(format!("Failed to copy raw wallpaper {}: {}", dir_name, e));
            } else {
                stats.raw_count += 1;
                log::success(&format!("Copied raw wallpaper: {}", dir_name));
            }
        }

    }
    
    log::success("Wallpaper extraction completed");
    Ok(stats)
}

fn check_has_pkg(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("pkg") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}
