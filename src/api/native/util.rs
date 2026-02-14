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
                !state.processed.contains_key(&w.wallpaper_id)
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
/// 将 project.json、preview 等文件从 Workshop 源目录复制到对应的 tex_converted 目录。
pub fn copy_metadata_to_tex_converted(config: &RuntimeConfig) {
    let workshop_path = &config.workshop_path;
    let unpacked_path = &config.unpacked_output_path;

    let entries = match fs::read_dir(unpacked_path) {
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

        let tex_dest_dir = wallpaper_dir.join("tex_converted");
        if !tex_dest_dir.exists() {
            continue;
        }

        let source_dir = workshop_path.join(&wallpaper_id);
        if !source_dir.exists() {
            continue;
        }

        // 基础元数据文件
        for filename in &["project.json", "scene.json"] {
            let src = source_dir.join(filename);
            if src.exists() {
                let dest = tex_dest_dir.join(filename);
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
                        let dest = tex_dest_dir.join(preview);
                        let _ = fs::copy(&src, &dest);
                    }
                }
            }
        }
    }
}

/// 清理 unpacked 目录（保留 tex_converted）
///
/// 删除 Pkg_Unpacked/壁纸ID/unpacked/ 子目录，保留 tex_converted。
pub fn clean_unpacked_dir(unpacked_path: &Path) {
    let entries = match fs::read_dir(unpacked_path) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let wallpaper_dir = entry.path();
        if !wallpaper_dir.is_dir() {
            continue;
        }

        let unpacked_sub = wallpaper_dir.join("unpacked");
        if unpacked_sub.exists() {
            let _ = fs::remove_dir_all(&unpacked_sub);
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
/// - 有自定义输出路径：平坦放入 custom_output
/// - 无自定义输出：放入 `Pkg_Unpacked/<wallpaper_id>/tex_converted/`——所有 TEX 扁平存放
pub fn determine_tex_output_path(
    tex_path: &Path,
    unpacked_path: &Path,
    custom_output: &Option<PathBuf>,
) -> PathBuf {
    let file_stem = tex_path.file_stem().unwrap_or_default();

    match custom_output {
        Some(output_base) => {
            let _ = crate::core::path::ensure_dir_compat(output_base);
            output_base.join(file_stem)
        }
        None => {
            // 从 tex_path 中提取 wallpaper_id（unpacked_path 后的第一级目录）
            let wallpaper_dir = if let Ok(relative) = tex_path.strip_prefix(unpacked_path) {
                if let Some(first_component) = relative.components().next() {
                    unpacked_path.join(first_component.as_os_str())
                } else {
                    tex_path.parent().unwrap_or(unpacked_path).to_path_buf()
                }
            } else {
                tex_path.parent().unwrap_or(unpacked_path).to_path_buf()
            };

            let output_dir = wallpaper_dir.join("tex_converted");
            let _ = crate::core::path::ensure_dir_compat(&output_dir);
            output_dir.join(file_stem)
        }
    }
}
