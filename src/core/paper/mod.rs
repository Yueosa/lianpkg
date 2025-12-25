use std::fs;
use std::path::Path;
use crate::core::path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WallpaperStats {
    pub raw_count: usize,
    pub pkg_count: usize,
}

pub fn extract_wallpapers(search_path: &Path, raw_output: &Path, pkg_temp_output: &Path, enable_raw: bool) -> Result<WallpaperStats, String> {
    let mut stats = WallpaperStats { raw_count: 0, pkg_count: 0 };
    
    if enable_raw {
        if let Err(e) = fs::create_dir_all(raw_output) {
            return Err(format!("Failed to create raw output dir: {}", e));
        }
    }
    if let Err(e) = fs::create_dir_all(pkg_temp_output) {
        return Err(format!("Failed to create pkg temp dir: {}", e));
    }

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
            if let Ok(sub_entries) = fs::read_dir(&path) {
                for sub_entry in sub_entries.flatten() {
                    let sub_path = sub_entry.path();
                    if let Some(ext) = sub_path.extension().and_then(|s| s.to_str()) {
                        if ext.eq_ignore_ascii_case("pkg") {
                            let file_name = sub_path.file_name().unwrap().to_str().unwrap();
                            let new_name = path::pkg_temp_dest(dir_name, file_name);
                            let dest = pkg_temp_output.join(new_name);
                            
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
            if !enable_raw {
                continue;
            }
            let dest_dir = raw_output.join(dir_name);
            if dest_dir.exists() {
                continue;
            }

            if let Err(e) = copy_dir_recursive(&path, &dest_dir) {
                return Err(format!("Failed to copy raw wallpaper {}: {}", dir_name, e));
            } else {
                stats.raw_count += 1;
            }
        }

    }
    
    Ok(stats)
}

pub fn estimate_requirements(search_path: &Path, enable_raw: bool) -> (u64, u64) {
    let mut pkg_size = 0;
    let mut raw_size = 0;

    if let Ok(entries) = fs::read_dir(search_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }

            if check_has_pkg(&path) {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub in sub_entries.flatten() {
                        let p = sub.path();
                        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                            if ext.eq_ignore_ascii_case("pkg") {
                                if let Ok(meta) = fs::metadata(&p) {
                                    pkg_size += meta.len();
                                }
                            }
                        }
                    }
                }
            } else if enable_raw {
                raw_size += get_dir_size(&path);
            }
        }
    }
    (pkg_size, raw_size)
}

fn get_dir_size(path: &Path) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                size += get_dir_size(&p);
            } else {
                if let Ok(meta) = fs::metadata(&p) {
                    size += meta.len();
                }
            }
        }
    }
    size
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
