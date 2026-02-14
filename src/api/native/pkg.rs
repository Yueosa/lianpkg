//! PKG 处理接口
//!
//! 封装 core::pkg 的底层操作，提供批量解包、预览等功能。
//! 所有接口返回 `CoreResult<T>`。

use crate::core::{error::CoreResult, path, pkg};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ============================================================================
// 类型定义
// ============================================================================

/// PKG 文件来源（壁纸 ID + 对应的 Workshop PKG 路径列表）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgSource {
    /// 壁纸 ID（Workshop 文件夹名）
    pub wallpaper_id: String,
    /// 该壁纸下的 PKG 文件路径列表
    pub pkg_paths: Vec<PathBuf>,
}

/// 批量解包输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnpackOutput {
    /// 解包结果列表
    pub results: Vec<UnpackResult>,
    /// 统计信息
    pub stats: UnpackStats,
}

/// 单个 PKG 解包结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnpackResult {
    /// PKG 文件路径
    pub pkg_path: PathBuf,
    /// PKG 文件名
    pub pkg_name: String,
    /// 场景名称
    pub scene_name: String,
    /// 输出目录
    pub output_dir: PathBuf,
    /// 是否成功
    pub success: bool,
    /// 解包的文件信息
    pub files: Vec<UnpackedFile>,
    /// 错误信息
    pub error: Option<String>,
}

/// 解包后的文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnpackedFile {
    /// 文件名
    pub name: String,
    /// 输出路径
    pub output_path: PathBuf,
    /// 文件大小
    pub size: u32,
    /// 是否是 TEX 文件
    pub is_tex: bool,
}

/// 解包统计
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct UnpackStats {
    /// 处理的 PKG 文件数
    pub pkg_processed: usize,
    /// 成功解包数
    pub pkg_success: usize,
    /// 失败数
    pub pkg_failed: usize,
    /// 总解包文件数
    pub total_files: usize,
    /// TEX 文件数
    pub tex_files: usize,
}

/// PKG 预览信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgPreview {
    /// PKG 版本
    pub version: String,
    /// 文件数量
    pub file_count: u32,
    /// 文件列表
    pub files: Vec<PkgFileEntry>,
    /// TEX 文件数量
    pub tex_count: usize,
}

/// PKG 中的文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgFileEntry {
    /// 文件名
    pub name: String,
    /// 文件大小
    pub size: u32,
    /// 是否是 TEX 文件
    pub is_tex: bool,
}

// ============================================================================
// 接口
// ============================================================================

/// 批量解包 PKG 文件
///
/// 从 Workshop 源路径直接读取 PKG 并解包到输出目录。
pub fn unpack_all(
    pkg_sources: &[PkgSource],
    unpacked_output_path: &Path,
) -> CoreResult<UnpackOutput> {
    path::ensure_dir_compat(unpacked_output_path)
        .map_err(|e| crate::core::error::CoreError::io(e))?;

    let mut results = Vec::new();
    let mut stats = UnpackStats::default();

    for source in pkg_sources {
        let output_dir = unpacked_output_path.join(&source.wallpaper_id).join("unpacked");

        for pkg_path in &source.pkg_paths {
            stats.pkg_processed += 1;

            let pkg_name = pkg_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            match pkg::unpack_pkg(pkg::UnpackPkgInput {
                file_path: pkg_path.clone(),
                output_base: output_dir.clone(),
            }) {
                Ok(result) => {
                    stats.pkg_success += 1;
                    let files: Vec<UnpackedFile> = result
                        .extracted_files
                        .iter()
                        .map(|f| {
                            let is_tex = f.entry_name.to_lowercase().ends_with(".tex");
                            if is_tex {
                                stats.tex_files += 1;
                            }
                            stats.total_files += 1;
                            UnpackedFile {
                                name: f.entry_name.clone(),
                                output_path: f.output_path.clone(),
                                size: f.size,
                                is_tex,
                            }
                        })
                        .collect();

                    results.push(UnpackResult {
                        pkg_path: pkg_path.clone(),
                        pkg_name,
                        scene_name: source.wallpaper_id.clone(),
                        output_dir: output_dir.clone(),
                        success: true,
                        files,
                        error: None,
                    });
                }
                Err(e) => {
                    stats.pkg_failed += 1;
                    results.push(UnpackResult {
                        pkg_path: pkg_path.clone(),
                        pkg_name,
                        scene_name: source.wallpaper_id.clone(),
                        output_dir: output_dir.clone(),
                        success: false,
                        files: vec![],
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    Ok(UnpackOutput { results, stats })
}

/// 解包单个 PKG 文件
pub fn unpack_single(pkg_path: &Path, output_base: &Path) -> CoreResult<UnpackResult> {
    let pkg_name = pkg_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let scene_name = path::scene_name_from_pkg_stem(
        pkg_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
            .as_str(),
    );

    let output_dir = output_base.join(&scene_name).join("unpacked");

    match pkg::unpack_pkg(pkg::UnpackPkgInput {
        file_path: pkg_path.to_path_buf(),
        output_base: output_dir.clone(),
    }) {
        Ok(result) => {
            let files: Vec<UnpackedFile> = result
                .extracted_files
                .iter()
                .map(|f| UnpackedFile {
                    name: f.entry_name.clone(),
                    output_path: f.output_path.clone(),
                    size: f.size,
                    is_tex: f.entry_name.to_lowercase().ends_with(".tex"),
                })
                .collect();

            Ok(UnpackResult {
                pkg_path: pkg_path.to_path_buf(),
                pkg_name,
                scene_name,
                output_dir,
                success: true,
                files,
                error: None,
            })
        }
        Err(e) => Ok(UnpackResult {
            pkg_path: pkg_path.to_path_buf(),
            pkg_name,
            scene_name,
            output_dir,
            success: false,
            files: vec![],
            error: Some(e.to_string()),
        }),
    }
}

/// 预览 PKG 文件内容（不解包）
pub fn preview_pkg(pkg_path: &Path) -> CoreResult<PkgPreview> {
    let pkg_info = pkg::parse_pkg(pkg::ParsePkgInput {
        file_path: pkg_path.to_path_buf(),
    })?
    .pkg_info;

    let files: Vec<PkgFileEntry> = pkg_info
        .entries
        .iter()
        .map(|e| PkgFileEntry {
            name: e.name.clone(),
            size: e.size,
            is_tex: e.name.to_lowercase().ends_with(".tex"),
        })
        .collect();

    let tex_count = files.iter().filter(|f| f.is_tex).count();

    Ok(PkgPreview {
        version: pkg_info.version,
        file_count: pkg_info.file_count,
        files,
        tex_count,
    })
}

/// 获取解包目录下的所有 TEX 文件（递归）
pub fn get_tex_files_from_unpacked(unpacked_path: &Path) -> Vec<PathBuf> {
    super::util::find_tex_files(unpacked_path)
}
