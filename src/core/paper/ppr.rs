use std::fs;
use std::path::Path;

use crate::core::paper::structs::{WallpaperStats, ProjectMeta, FolderProcess};
use crate::core::paper::utl::{copy_dir_recursive, check_has_pkg};
use crate::core::path;

pub fn list_workshop_dirs(search_path: &Path) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(search_path)
        .map_err(|e| format!("Failed to read search path: {}", e))?;

    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                dirs.push(name.to_string());
            }
        }
    }
    Ok(dirs)
}

pub fn read_project_meta(folder: &Path) -> Result<ProjectMeta, String> {
    let meta_path = folder.join("project.json");
    if !meta_path.exists() {
        return Err("project.json not found".to_string());
    }
    let content = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read {}: {}", meta_path.display(), e))?;
    serde_json::from_str::<ProjectMeta>(&content)
        .map_err(|e| format!("Failed to parse {}: {}", meta_path.display(), e))
}

pub fn process_folder(folder: &Path, raw_output: &Path, pkg_temp_output: &Path, enable_raw: bool) -> Result<FolderProcess, String> {
    let dir_name = folder
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid folder name".to_string())?;

    let has_pkg = check_has_pkg(folder);
    let mut result = FolderProcess::default();

    if has_pkg {
        if let Ok(sub_entries) = fs::read_dir(folder) {
            for sub_entry in sub_entries.flatten() {
                let sub_path = sub_entry.path();
                if let Some(ext) = sub_path.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("pkg") {
                        let file_name = sub_path.file_name().unwrap().to_str().unwrap();
                        let new_name = path::pkg_temp_dest(dir_name, file_name);
                        let dest = pkg_temp_output.join(new_name);
                        fs::create_dir_all(pkg_temp_output)
                            .map_err(|e| format!("Failed to create pkg temp dir: {}", e))?;
                        if let Err(e) = fs::copy(&sub_path, &dest) {
                            return Err(format!("Failed to copy pkg: {}", e));
                        } else {
                            result.copied_pkgs += 1;
                        }
                    }
                }
            }
        }
    } else if enable_raw {
        let dest_dir = raw_output.join(dir_name);
        if dest_dir.exists() {
            return Ok(result);
        }
        fs::create_dir_all(raw_output)
            .map_err(|e| format!("Failed to create raw output dir: {}", e))?;
        if let Err(e) = copy_dir_recursive(folder, &dest_dir) {
            return Err(format!("Failed to copy raw wallpaper {}: {}", dir_name, e));
        } else {
            result.copied_raw = true;
        }
    }

    Ok(result)
}

pub fn extract_wallpapers(search_path: &Path, raw_output: &Path, pkg_temp_output: &Path, enable_raw: bool) -> Result<WallpaperStats, String> {
    let mut stats = WallpaperStats { raw_count: 0, pkg_count: 0 };

    let entries = fs::read_dir(search_path)
        .map_err(|e| format!("Failed to read search path: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let res = process_folder(&path, raw_output, pkg_temp_output, enable_raw)?;
        if res.copied_raw { stats.raw_count += 1; }
        stats.pkg_count += res.copied_pkgs;
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
                raw_size += crate::core::paper::utl::get_dir_size(&path);
            }
        }
    }
    (pkg_size, raw_size)
}
