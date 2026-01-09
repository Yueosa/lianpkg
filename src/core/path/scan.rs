//! 文件扫描

use super::types::*;
use crate::core::error::CoreResult;
use std::fs;
use std::path::{Path, PathBuf};

/// 扫描目标文件
///
/// 支持文件或目录输入，递归扫描指定扩展名的文件
pub fn scan_files(input: ScanFilesInput) -> CoreResult<ScanFilesOutput> {
    let mut files = Vec::new();

    let extensions: Vec<String> = input
        .extensions
        .unwrap_or_else(|| vec!["pkg".to_string(), "tex".to_string()]);

    if input.path.is_file() {
        // 单文件：检查扩展名
        if let Some(ext) = input.path.extension() {
            let ext_str: String = ext.to_string_lossy().to_lowercase();
            if extensions
                .iter()
                .any(|e: &String| e.to_lowercase() == ext_str)
            {
                files.push(input.path);
            }
        }
    } else if input.path.is_dir() {
        visit_dirs(&input.path, &mut files, &extensions);
    }

    Ok(ScanFilesOutput { files })
}

/// 递归遍历目录，收集指定扩展名的文件
fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>, extensions: &[String]) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                visit_dirs(&path, files, extensions);
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                    files.push(path);
                }
            }
        }
    }
}
