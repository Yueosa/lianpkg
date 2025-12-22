use std::path::{Path, PathBuf};
use std::fs;
use crate::log;

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
