use std::path::{Path, PathBuf};
use std::fs;
use crate::log;
use crate::config::{self, Config};

/// Resolves the final output directory for converted textures and resources.
/// 
/// Logic:
/// 1. If `config.tex.converted_output_path` is set:
///    - Base = expand(custom_path) / "tex_converted"
///    - If `input_file` is provided, we try to maintain relative structure from `root_path`.
/// 2. If not set:
///    - Base = `root_path` / "tex_converted"
///    - Maintains relative structure inside.
pub fn resolve_tex_output_dir(config: &Config, root_path: &Path, input_file: Option<&Path>, relative_base: Option<&Path>) -> PathBuf {
    let base_dir = if let Some(custom_path) = &config.tex.converted_output_path {
        config::expand_path(custom_path).join("tex_converted")
    } else {
        root_path.join("tex_converted")
    };

    // If we have an input file and a base to calculate relativity from
    if let (Some(file), Some(base)) = (input_file, relative_base) {
        // Try to keep relative path structure
        if let Ok(relative) = file.strip_prefix(base) {
            // If using custom output path, we need to prepend the scene directory name
            // because 'base' is usually the scene root (e.g. .../123456_scene)
            // and 'relative' is inside it (e.g. materials/a.tex).
            // We want: custom_path/tex_converted/123456_scene/materials/
            
            if config.tex.converted_output_path.is_some() {
                if let Some(scene_dir_name) = base.file_name() {
                    if let Some(parent) = relative.parent() {
                        return base_dir.join(scene_dir_name).join(parent);
                    }
                }
            }

            if let Some(parent) = relative.parent() {
                return base_dir.join(parent);
            }
        }
    } 
    // If we don't have a specific input file (e.g. copying resources for a whole folder),
    // but we have a relative_base (which is likely the root of the package/scene),
    // and root_path is the specific scene folder.
    // We need to append the scene folder name to the base_dir if we are using a custom output path.
    else if config.tex.converted_output_path.is_some() {
         // When using custom output, we need to distinguish different scenes.
         // Usually 'root_path' points to ".../Pkg_Unpacked/123456_scene"
         // We want the result to be ".../tex_converted/123456_scene"
         if let Some(dir_name) = root_path.file_name() {
             return base_dir.join(dir_name);
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
