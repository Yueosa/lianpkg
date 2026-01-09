//! 扫描相关接口 - 目录列举、元数据读取、pkg检查、空间估算

use std::fs;

use crate::core::error::{CoreError, CoreResult};
use crate::core::paper::structs::{
    CheckPkgInput, CheckPkgOutput, EstimateInput, EstimateOutput, ListDirsInput, ListDirsOutput,
    ProjectMeta, ReadMetaInput, ReadMetaOutput,
};
use crate::core::paper::utl::get_dir_size;

/// 列出指定目录下的所有子目录
pub fn list_dirs(input: ListDirsInput) -> CoreResult<ListDirsOutput> {
    let path = input.path;

    let entries = fs::read_dir(&path).map_err(|e| CoreError::Io {
        message: e.to_string(),
        path: Some(path.display().to_string()),
    })?;

    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                dirs.push(name.to_string());
            }
        }
    }

    Ok(ListDirsOutput { dirs })
}

/// 读取壁纸文件夹的 project.json 元数据
pub fn read_meta(input: ReadMetaInput) -> CoreResult<ReadMetaOutput> {
    let meta_path = input.folder.join("project.json");

    if !meta_path.exists() {
        return Err(CoreError::NotFound {
            message: "project.json not found".to_string(),
            path: Some(meta_path.display().to_string()),
        });
    }

    let content = fs::read_to_string(&meta_path).map_err(|e| CoreError::Io {
        message: e.to_string(),
        path: Some(meta_path.display().to_string()),
    })?;

    let meta: ProjectMeta = serde_json::from_str(&content).map_err(|e| CoreError::Parse {
        message: e.to_string(),
        source: Some(meta_path.display().to_string()),
    })?;

    Ok(ReadMetaOutput { meta })
}

/// 检查文件夹是否包含 .pkg 文件
pub fn check_pkg(input: CheckPkgInput) -> CheckPkgOutput {
    let folder = input.folder;
    let mut pkg_files = Vec::new();

    if let Ok(entries) = fs::read_dir(&folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("pkg") {
                        pkg_files.push(path);
                    }
                }
            }
        }
    }

    CheckPkgOutput {
        has_pkg: !pkg_files.is_empty(),
        pkg_files,
    }
}

/// 估算处理所需的磁盘空间
pub fn estimate(input: EstimateInput) -> EstimateOutput {
    let search_path = input.search_path;
    let enable_raw = input.enable_raw;

    let mut pkg_size: u64 = 0;
    let mut raw_size: u64 = 0;
    let mut pkg_count: usize = 0;
    let mut raw_count: usize = 0;

    if let Ok(entries) = fs::read_dir(&search_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // 检查是否有 pkg 文件
            let check_result = check_pkg(CheckPkgInput {
                folder: path.clone(),
            });

            if check_result.has_pkg {
                pkg_count += 1;
                for pkg_path in &check_result.pkg_files {
                    if let Ok(meta) = fs::metadata(pkg_path) {
                        pkg_size += meta.len();
                    }
                }
            } else if enable_raw {
                raw_count += 1;
                raw_size += get_dir_size(&path);
            }
        }
    }

    EstimateOutput {
        pkg_size,
        raw_size,
        pkg_count,
        raw_count,
    }
}
