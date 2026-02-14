//! 壁纸扫描与复制
//!
//! 提供 Workshop 壁纸扫描、详情查询、复制等功能。
//! 扫描结果中包含增量状态标记（is_processed），避免重复扫描。

use crate::core::{cfg, error::CoreResult, paper};
use super::context::AppContext;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// 扫描类型
// ============================================================================

/// 扫描输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOutput {
    /// 壁纸列表
    pub wallpapers: Vec<WallpaperInfo>,
    /// 统计信息
    pub stats: ScanStats,
}

/// 壁纸信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperInfo {
    /// 壁纸 ID（文件夹名）
    pub wallpaper_id: String,
    /// 壁纸标题
    pub title: Option<String>,
    /// 壁纸类型（scene/video/web 等）
    pub wallpaper_type: Option<String>,
    /// 预览图路径
    pub preview_path: Option<PathBuf>,
    /// 是否包含 pkg 文件
    pub has_pkg: bool,
    /// pkg 文件列表
    pub pkg_files: Vec<PathBuf>,
    /// 文件夹路径
    pub folder_path: PathBuf,
    /// 是否已处理（增量模式参考）
    pub is_processed: bool,
}

/// 扫描统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanStats {
    /// 总壁纸数
    pub total_count: usize,
    /// 包含 pkg 的壁纸数
    pub pkg_count: usize,
    /// 原始壁纸数（不含 pkg）
    pub raw_count: usize,
}

// ============================================================================
// 复制类型
// ============================================================================

/// 复制输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyOutput {
    /// 复制结果列表
    pub results: Vec<CopyResult>,
    /// 统计信息
    pub stats: CopyStats,
}

/// 单个壁纸复制结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyResult {
    /// 壁纸 ID
    pub wallpaper_id: String,
    /// 壁纸标题
    pub title: Option<String>,
    /// 处理类型
    pub result_type: CopyResultType,
    /// PKG 文件路径列表
    pub pkg_files: Vec<PathBuf>,
}

/// 复制结果类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CopyResultType {
    /// 复制为原始壁纸
    Raw,
    /// 包含 PKG 文件
    Pkg,
    /// 跳过
    Skipped,
}

/// 复制统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopyStats {
    /// 原始壁纸复制数
    pub raw_copied: usize,
    /// Pkg 壁纸数
    pub pkg_copied: usize,
    /// 跳过数
    pub skipped: usize,
    /// 总 pkg 文件数
    pub total_pkg_files: usize,
}

// ============================================================================
// 扫描接口
// ============================================================================

/// 使用 AppContext 扫描壁纸（自动加载状态，标记 is_processed）
pub fn scan(ctx: &AppContext) -> CoreResult<ScanOutput> {
    let state = super::context::load_state_or_default(&ctx.state_path);
    scan_workshop(&ctx.config.workshop_path, Some(&state))
}

/// 扫描 Workshop 目录
///
/// `state` 为 Some 时会标记每个壁纸的 `is_processed` 字段。
pub fn scan_workshop(
    workshop_path: &std::path::Path,
    state: Option<&cfg::StateData>,
) -> CoreResult<ScanOutput> {
    let list_result = paper::list_dirs(paper::ListDirsInput {
        path: workshop_path.to_path_buf(),
    })?;

    let mut wallpapers = Vec::new();
    let mut stats = ScanStats::default();

    for dir_name in list_result.dirs {
        let folder_path = workshop_path.join(&dir_name);

        // 读取元数据（失败时使用默认值）
        let (title, wallpaper_type, preview_path) = match paper::read_meta(paper::ReadMetaInput {
            folder: folder_path.clone(),
        }) {
            Ok(r) => (
                r.meta.title,
                r.meta.wallpaper_type,
                r.meta.preview.map(|p| folder_path.join(p)),
            ),
            Err(_) => (None, None, None),
        };

        // 检查 pkg 文件
        let pkg_result = paper::check_pkg(paper::CheckPkgInput {
            folder: folder_path.clone(),
        });

        let is_processed = state
            .map(|s| s.processed.contains_key(&dir_name))
            .unwrap_or(false);

        let info = WallpaperInfo {
            wallpaper_id: dir_name,
            title,
            wallpaper_type,
            preview_path,
            has_pkg: pkg_result.has_pkg,
            pkg_files: pkg_result.pkg_files,
            folder_path,
            is_processed,
        };

        stats.total_count += 1;
        if info.has_pkg {
            stats.pkg_count += 1;
        } else {
            stats.raw_count += 1;
        }

        wallpapers.push(info);
    }

    Ok(ScanOutput { wallpapers, stats })
}

/// 获取单个壁纸详情
pub fn get_wallpaper_detail(
    workshop_path: &std::path::Path,
    wallpaper_id: &str,
) -> Option<WallpaperInfo> {
    let folder_path = workshop_path.join(wallpaper_id);

    if !folder_path.exists() || !folder_path.is_dir() {
        return None;
    }

    let (title, wallpaper_type, preview_path) = match paper::read_meta(paper::ReadMetaInput {
        folder: folder_path.clone(),
    }) {
        Ok(r) => (
            r.meta.title,
            r.meta.wallpaper_type,
            r.meta.preview.map(|p| folder_path.join(p)),
        ),
        Err(_) => (None, None, None),
    };

    let pkg_result = paper::check_pkg(paper::CheckPkgInput {
        folder: folder_path.clone(),
    });

    Some(WallpaperInfo {
        wallpaper_id: wallpaper_id.to_string(),
        title,
        wallpaper_type,
        preview_path,
        has_pkg: pkg_result.has_pkg,
        pkg_files: pkg_result.pkg_files,
        folder_path,
        is_processed: false,
    })
}

// ============================================================================
// 复制接口
// ============================================================================

/// 复制壁纸到目标目录
///
/// 接受预扫描的壁纸列表，避免重复扫描 Workshop 目录。
/// 可通过传入经过筛选的子集来实现增量处理或按 ID 过滤。
pub fn copy_wallpapers(
    wallpapers: &[WallpaperInfo],
    raw_output_path: &std::path::Path,
    enable_raw: bool,
) -> CoreResult<CopyOutput> {
    let mut results = Vec::new();
    let mut stats = CopyStats::default();

    for wallpaper in wallpapers {
        let process_result = paper::process_folder(paper::ProcessFolderInput {
            folder: wallpaper.folder_path.clone(),
            raw_output: raw_output_path.to_path_buf(),
            enable_raw,
        });

        let result_type = match process_result.result_type {
            paper::ProcessResultType::Raw => {
                stats.raw_copied += 1;
                CopyResultType::Raw
            }
            paper::ProcessResultType::Pkg => {
                stats.pkg_copied += 1;
                stats.total_pkg_files += process_result.pkg_files.len();
                CopyResultType::Pkg
            }
            paper::ProcessResultType::Skipped => {
                stats.skipped += 1;
                CopyResultType::Skipped
            }
        };

        results.push(CopyResult {
            wallpaper_id: wallpaper.wallpaper_id.clone(),
            title: wallpaper.title.clone(),
            result_type,
            pkg_files: process_result.pkg_files,
        });
    }

    Ok(CopyOutput { results, stats })
}

/// 复制壁纸（便捷版：自动扫描 + 按 ID 过滤）
///
/// 用于 CLI wallpaper 命令等需要独立调用的场景。
pub fn scan_and_copy(
    workshop_path: &std::path::Path,
    wallpaper_ids: Option<&[String]>,
    raw_output_path: &std::path::Path,
    enable_raw: bool,
) -> CoreResult<CopyOutput> {
    let scan_result = scan_workshop(workshop_path, None)?;

    let wallpapers: Vec<&WallpaperInfo> = match wallpaper_ids {
        Some(ids) => scan_result
            .wallpapers
            .iter()
            .filter(|w| ids.contains(&w.wallpaper_id))
            .collect(),
        None => scan_result.wallpapers.iter().collect(),
    };

    // 转为切片引用
    let owned: Vec<WallpaperInfo> = wallpapers.into_iter().cloned().collect();
    copy_wallpapers(&owned, raw_output_path, enable_raw)
}
