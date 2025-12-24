use std::path::{Path, PathBuf};
use std::fs;
use crate::log;

pub fn expand_path(path_str: &str) -> PathBuf {
    if path_str.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            if path_str == "~" {
                return home;
            }
            if path_str.starts_with("~/") {
                return home.join(&path_str[2..]);
            }
        }
    }
    PathBuf::from(path_str)
}

fn get_steam_base_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey("Software\\Valve\\Steam")
            .ok()
            .and_then(|steam| steam.get_value::<String, _>("SteamPath").ok())
            .map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let p = expand_path("~/.local/share/Steam");
        if p.exists() { Some(p) } else { None }
    }
}

fn find_library_path(steam_base: &Path) -> Option<PathBuf> {
    let vdf_path = steam_base.join("steamapps").join("libraryfolders.vdf");
    if !vdf_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&vdf_path).ok()?;
    let mut current_path = None;
    
    for line in content.lines() {
        let line = line.trim();
        // Match "path" "..."
        if line.starts_with("\"path\"") {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 4 {
                let p = parts[3].replace("\\\\", "\\");
                current_path = Some(PathBuf::from(p));
            }
        }
        // Match "431960"
        if line.contains("\"431960\"") {
            return current_path;
        }
    }
    None
}

pub fn default_workshop_path() -> String {
    if let Some(base_path) = get_steam_base_path() {
        // Try to find actual library path from vdf
        if let Some(lib_path) = find_library_path(&base_path) {
             return lib_path
                .join("steamapps")
                .join("workshop")
                .join("content")
                .join("431960")
                .to_string_lossy()
                .to_string();
        }
        
        // Fallback to default steam library
        return base_path
            .join("steamapps")
            .join("workshop")
            .join("content")
            .join("431960")
            .to_string_lossy()
            .to_string();
    }

    #[cfg(target_os = "windows")]
    {
        r"C:\Program Files (x86)\Steam\steamapps\workshop\content\431960".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    {
        "~/.local/share/Steam/steamapps/workshop/content/431960".to_string()
    }
}

pub fn default_raw_output_path() -> String {
    #[cfg(target_os = "windows")]
    { r".\\Wallpapers_Raw".to_string() }
    #[cfg(not(target_os = "windows"))]
    { "~/.local/share/lianpkg/Wallpapers_Raw".to_string() }
}

pub fn default_pkg_temp_path() -> String {
    #[cfg(target_os = "windows")]
    { r".\\Pkg_Temp".to_string() }
    #[cfg(not(target_os = "windows"))]
    { "~/.local/share/lianpkg/Pkg_Temp".to_string() }
}

pub fn default_unpacked_output_path() -> String {
    #[cfg(target_os = "windows")]
    { r".\\Pkg_Unpacked".to_string() }
    #[cfg(not(target_os = "windows"))]
    { "~/.local/share/lianpkg/Pkg_Unpacked".to_string() }
}

pub fn pkg_temp_dest(dir_name: &str, file_name: &str) -> String {
    format!("{}_{}", dir_name, file_name)
}

pub fn scene_name_from_pkg_stem(stem: &str) -> String {
    if let Some((prefix, _)) = stem.split_once('_') {
        prefix.to_string()
    } else {
        stem.to_string()
    }
}

pub fn resolve_tex_output_dir(
    converted_output_path: Option<&str>,
    scene_root: &Path,
    input_file: Option<&Path>,
    relative_base: Option<&Path>,
) -> PathBuf {
    let base_dir = if let Some(custom_path) = converted_output_path {
        expand_path(custom_path)
            .join("tex_converted")
            .join(scene_root.file_name().unwrap_or_default())
    } else {
        scene_root.join("tex_converted")
    };

    if let (Some(file), Some(base)) = (input_file, relative_base) {
        if let Ok(relative) = file.strip_prefix(base) {
            if let Some(parent) = relative.parent() {
                return base_dir.join(parent);
            }
        }
    } 
    base_dir
}

pub fn get_target_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    log::debug("get_target_files", &format!("{:?}", path), "Scanning for files...");

    if path.is_file() {
        files.push(path.to_path_buf());
        log::debug("get_target_files", "N/A", "Input is a single file");
    } else if path.is_dir() {
        visit_dirs(path, &mut files);
        log::debug("get_target_files", "N/A", &format!("Found {} files in directory", files.len()));
    } else {
        log::info(&format!("Path does not exist or is not accessible: {:?}", path));
    }
    files
}

fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files);
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "pkg" || ext_str == "tex" {
                    files.push(path);
                }
            }
        }
    }
}

pub fn find_project_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(p) = current {
        if p.join("project.json").exists() || p.join("scene.json").exists() {
            return Some(p.to_path_buf());
        }
        
        if p.join("materials").is_dir() {
            if path.starts_with(p.join("materials")) {
                return Some(p.to_path_buf());
            }
        }

        if p.parent().is_none() {
            break;
        }
        current = p.parent();
    }
    None
}
