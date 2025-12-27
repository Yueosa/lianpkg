//! 复制相关接口 - 单文件夹处理、批量提取

use std::fs;

use crate::core::paper::structs::{
    ProcessFolderInput, ProcessFolderOutput,
    ExtractInput, ExtractOutput,
    ProcessResultType, ProcessedFolder, WallpaperStats,
    CheckPkgInput,
};
use crate::core::paper::scan::check_pkg;
use crate::core::paper::utl::copy_dir_recursive;
use crate::core::path;

/// 处理单个壁纸文件夹
/// 根据是否包含 pkg 文件决定复制方式
pub fn process_folder(input: ProcessFolderInput) -> ProcessFolderOutput {
    let folder = &input.folder;
    let raw_output = &input.raw_output;
    let pkg_temp_output = &input.pkg_temp_output;
    let enable_raw = input.enable_raw;

    // 获取文件夹名称
    let dir_name = match folder.file_name().and_then(|n| n.to_str()) {
        Some(name) => name.to_string(),
        None => {
            return ProcessFolderOutput {
                copied_raw: false,
                copied_pkgs: 0,
                skipped: true,
                result_type: ProcessResultType::Skipped,
                pkg_files: Vec::new(),
            };
        }
    };

    // 检查是否有 pkg 文件
    let check_result = check_pkg(CheckPkgInput { folder: folder.clone() });

    if check_result.has_pkg {
        // 有 pkg 文件，复制 pkg 到临时目录
        let mut copied_pkgs = 0;
        let mut copied_files = Vec::new();

        // 确保目标目录存在
        if fs::create_dir_all(pkg_temp_output).is_err() {
            return ProcessFolderOutput {
                copied_raw: false,
                copied_pkgs: 0,
                skipped: true,
                result_type: ProcessResultType::Skipped,
                pkg_files: Vec::new(),
            };
        }

        for pkg_path in &check_result.pkg_files {
            if let Some(file_name) = pkg_path.file_name().and_then(|n| n.to_str()) {
                let new_name = path::pkg_temp_dest(&dir_name, file_name);
                let dest = pkg_temp_output.join(&new_name);
                
                if fs::copy(pkg_path, &dest).is_ok() {
                    copied_pkgs += 1;
                    copied_files.push(dest);
                }
            }
        }

        ProcessFolderOutput {
            copied_raw: false,
            copied_pkgs,
            skipped: copied_pkgs == 0,
            result_type: if copied_pkgs > 0 { ProcessResultType::Pkg } else { ProcessResultType::Skipped },
            pkg_files: copied_files,
        }
    } else if enable_raw {
        // 无 pkg 文件，复制整个目录作为原始壁纸
        let dest_dir = raw_output.join(&dir_name);

        // 如果目标已存在，跳过
        if dest_dir.exists() {
            return ProcessFolderOutput {
                copied_raw: false,
                copied_pkgs: 0,
                skipped: true,
                result_type: ProcessResultType::Skipped,
                pkg_files: Vec::new(),
            };
        }

        // 确保父目录存在
        if fs::create_dir_all(raw_output).is_err() {
            return ProcessFolderOutput {
                copied_raw: false,
                copied_pkgs: 0,
                skipped: true,
                result_type: ProcessResultType::Skipped,
                pkg_files: Vec::new(),
            };
        }

        // 递归复制目录
        if copy_dir_recursive(folder, &dest_dir).is_ok() {
            ProcessFolderOutput {
                copied_raw: true,
                copied_pkgs: 0,
                skipped: false,
                result_type: ProcessResultType::Raw,
                pkg_files: Vec::new(),
            }
        } else {
            ProcessFolderOutput {
                copied_raw: false,
                copied_pkgs: 0,
                skipped: true,
                result_type: ProcessResultType::Skipped,
                pkg_files: Vec::new(),
            }
        }
    } else {
        // 不启用原始壁纸提取，跳过
        ProcessFolderOutput {
            copied_raw: false,
            copied_pkgs: 0,
            skipped: true,
            result_type: ProcessResultType::Skipped,
            pkg_files: Vec::new(),
        }
    }
}

/// 批量提取壁纸
/// 遍历搜索路径下的所有文件夹并处理
pub fn extract_all(input: ExtractInput) -> ExtractOutput {
    let config = input.config;
    let mut stats = WallpaperStats::default();
    let mut processed_folders = Vec::new();

    let entries = match fs::read_dir(&config.search_path) {
        Ok(e) => e,
        Err(_) => {
            return ExtractOutput {
                stats,
                processed_folders,
            };
        }
    };

    for entry in entries.flatten() {
        let folder_path = entry.path();
        if !folder_path.is_dir() {
            continue;
        }

        let folder_name = folder_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // 处理单个文件夹
        let result = process_folder(ProcessFolderInput {
            folder: folder_path.clone(),
            raw_output: config.raw_output.clone(),
            pkg_temp_output: config.pkg_temp_output.clone(),
            enable_raw: config.enable_raw,
        });

        // 更新统计
        match result.result_type {
            ProcessResultType::Raw => stats.raw_count += 1,
            ProcessResultType::Pkg => stats.pkg_count += result.copied_pkgs,
            ProcessResultType::Skipped => {}
        }

        // 记录处理详情
        processed_folders.push(ProcessedFolder {
            folder_name,
            folder_path,
            result_type: result.result_type,
            pkg_files: result.pkg_files,
        });
    }

    ExtractOutput {
        stats,
        processed_folders,
    }
}
