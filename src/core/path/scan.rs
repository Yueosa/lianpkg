//! 文件扫描与项目定位

use std::fs;
use std::path::{Path, PathBuf};

/// 获取目标文件列表
/// 支持文件或目录输入，递归扫描 .pkg 和 .tex 文件
pub fn get_target_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        visit_dirs(path, &mut files);
    }
    
    files
}

/// 查找项目根目录
/// 向上遍历查找包含 project.json 或 scene.json 的目录
pub fn find_project_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    
    while let Some(p) = current {
        // 检查项目标识文件
        if p.join("project.json").exists() || p.join("scene.json").exists() {
            return Some(p.to_path_buf());
        }
        
        // 检查 materials 目录结构
        if p.join("materials").is_dir() {
            if path.starts_with(p.join("materials")) {
                return Some(p.to_path_buf());
            }
        }

        // 到达文件系统根目录
        if p.parent().is_none() {
            break;
        }
        
        current = p.parent();
    }
    
    None
}

/// 递归遍历目录，收集 .pkg 和 .tex 文件
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
