//! 内部工具函数
//!
//! 从 pipeline.rs 提取的公共工具，供 auto.rs 和其他模块使用。

use super::context::RuntimeConfig;
use super::scan::WallpaperInfo;
use crate::core::cfg;
use std::fs;
use std::path::{Path, PathBuf};

/// 筛选待处理的壁纸
///
/// 根据增量模式和指定 ID 列表过滤壁纸。
///
/// 增量模式下，只跳过已成功处理（Pkg/Raw）的壁纸，
/// 之前被 Skipped 的壁纸会重新评估（例如用户后来开启了 raw_output）。
pub fn filter_wallpapers(
    wallpapers: &[WallpaperInfo],
    state: &cfg::StateData,
    ids: Option<&Vec<String>>,
    incremental: bool,
) -> Vec<String> {
    wallpapers
        .iter()
        .filter(|w| {
            let in_list = match ids {
                Some(filter_ids) => filter_ids.contains(&w.wallpaper_id),
                None => true,
            };
            let not_processed = if incremental {
                match state.processed.get(&w.wallpaper_id) {
                    Some(info) => info.process_type == cfg::ProcessType::Skipped,
                    None => true,
                }
            } else {
                true
            };
            in_list && not_processed
        })
        .map(|w| w.wallpaper_id.clone())
        .collect()
}

/// 复制元数据文件到 tex_converted 目录
///
/// 将 project.json、preview 等文件从 Workshop 源目录复制到对应的 converted_output_path 目录。
pub fn copy_metadata_to_tex_converted(config: &RuntimeConfig) {
    let workshop_path = &config.workshop_path;
    let converted_path = &config.converted_output_path;

    let entries = match fs::read_dir(converted_path) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let wallpaper_dir = entry.path();
        if !wallpaper_dir.is_dir() {
            continue;
        }

        let wallpaper_id = match wallpaper_dir.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let source_dir = workshop_path.join(&wallpaper_id);
        if !source_dir.exists() {
            continue;
        }

        // 基础元数据文件
        for filename in &["project.json", "scene.json"] {
            let src = source_dir.join(filename);
            if src.exists() {
                let dest = wallpaper_dir.join(filename);
                let _ = fs::copy(&src, &dest);
            }
        }

        // 从 project.json 读取预览图文件名
        let project_path = source_dir.join("project.json");
        if let Ok(content) = fs::read_to_string(&project_path) {
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(preview) = meta.get("preview").and_then(|v| v.as_str()) {
                    let src = source_dir.join(preview);
                    if src.exists() {
                        let dest = wallpaper_dir.join(preview);
                        let _ = fs::copy(&src, &dest);
                    }
                }
            }
        }
    }
}

/// 清理 unpacked 目录
///
/// 删除 unpacked_output_path 下的所有内容（壁纸子目录），
/// converted_output_path 是独立目录，不受影响。
pub fn clean_unpacked_dir(unpacked_path: &Path) {
    let entries = match fs::read_dir(unpacked_path) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let _ = fs::remove_dir_all(&path);
        } else {
            let _ = fs::remove_file(&path);
        }
    }
}

/// 查找目录下所有 TEX 文件（递归，跳过 tex_converted 目录）
pub fn find_tex_files(dir: &Path) -> Vec<PathBuf> {
    let mut tex_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "tex" {
                        tex_files.push(path);
                    }
                }
            } else if path.is_dir() {
                // 跳过 tex_converted 目录，避免扫描已转换的输出
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name != "tex_converted" {
                    tex_files.extend(find_tex_files(&path));
                }
            }
        }
    }

    tex_files
}

/// 确定 TEX 输出路径
///
/// 输出规则：
/// - 从 tex_path 中提取相对于 unpacked_path 的 wallpaper_id（第一级目录）
/// - 输出到 converted_path/<wallpaper_id>/<file_stem>
pub fn determine_tex_output_path(
    tex_path: &Path,
    unpacked_path: &Path,
    converted_path: &Path,
) -> PathBuf {
    let file_stem = tex_path.file_stem().unwrap_or_default();

    // 从 tex_path 中提取 wallpaper_id（unpacked_path 后的第一级目录）
    let wallpaper_id = if let Ok(relative) = tex_path.strip_prefix(unpacked_path) {
        relative.components().next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
    } else {
        None
    };

    let output_dir = match wallpaper_id {
        Some(id) => converted_path.join(&id),
        None => converted_path.to_path_buf(),
    };

    let _ = crate::core::path::ensure_dir_compat(&output_dir);
    output_dir.join(file_stem)
}
