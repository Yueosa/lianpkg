//! 壁纸处理高级接口
//!
//! 封装 core::paper 的底层操作，提供更友好的 API。
//! 支持扫描、预览、复制等操作。

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::core::paper;

// ============================================================================
// 结构体定义
// ============================================================================

/// 扫描壁纸入参
#[derive(Debug, Clone)]
pub struct ScanWallpapersInput {
    /// Workshop 路径
    pub workshop_path: PathBuf,
}

/// 扫描壁纸返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanWallpapersOutput {
    /// 是否成功
    pub success: bool,
    /// 壁纸信息列表
    pub wallpapers: Vec<WallpaperInfo>,
    /// 统计信息
    pub stats: ScanStats,
    /// 错误信息
    pub error: Option<String>,
}

/// 壁纸信息（用于预览和选择）
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

/// 复制壁纸入参
#[derive(Debug, Clone)]
pub struct CopyWallpapersInput {
    /// 要复制的壁纸 ID 列表，None 表示全部
    pub wallpaper_ids: Option<Vec<String>>,
    /// Workshop 路径
    pub workshop_path: PathBuf,
    /// 原始壁纸输出路径
    pub raw_output_path: PathBuf,
    /// Pkg 临时输出路径
    pub pkg_temp_path: PathBuf,
    /// 是否复制原始壁纸
    pub enable_raw: bool,
}

/// 复制壁纸返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyWallpapersOutput {
    /// 是否成功
    pub success: bool,
    /// 复制结果列表
    pub results: Vec<CopyResult>,
    /// 统计信息
    pub stats: CopyStats,
    /// 错误信息
    pub error: Option<String>,
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
    /// 复制的 pkg 文件路径
    pub pkg_files: Vec<PathBuf>,
}

/// 复制结果类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CopyResultType {
    /// 复制为原始壁纸
    Raw,
    /// 复制了 pkg 文件
    Pkg,
    /// 跳过
    Skipped,
}

/// 复制统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopyStats {
    /// 原始壁纸复制数
    pub raw_copied: usize,
    /// Pkg 壁纸复制数
    pub pkg_copied: usize,
    /// 跳过数
    pub skipped: usize,
    /// 总 pkg 文件数
    pub total_pkg_files: usize,
}

// ============================================================================
// 接口实现
// ============================================================================

/// 扫描 Workshop 目录下的所有壁纸
/// 
/// 返回壁纸列表及其基本信息，用于预览和选择
pub fn scan_wallpapers(input: ScanWallpapersInput) -> ScanWallpapersOutput {
    // 列出所有目录
    let list_result = paper::list_dirs(paper::ListDirsInput {
        path: input.workshop_path.clone(),
    });

    if !list_result.success {
        return ScanWallpapersOutput {
            success: false,
            wallpapers: vec![],
            stats: ScanStats::default(),
            error: Some("Failed to list wallpaper directories".to_string()),
        };
    }

    let mut wallpapers = Vec::new();
    let mut stats = ScanStats::default();

    for dir_name in list_result.dirs {
        let folder_path = input.workshop_path.join(&dir_name);

        // 读取元数据
        let meta_result = paper::read_meta(paper::ReadMetaInput {
            folder: folder_path.clone(),
        });

        let (title, wallpaper_type, preview_path) = if meta_result.success {
            let meta = meta_result.meta.unwrap_or_default();
            (
                meta.title,
                meta.wallpaper_type,
                meta.preview.map(|p| folder_path.join(p)),
            )
        } else {
            (None, None, None)
        };

        // 检查 pkg 文件
        let pkg_result = paper::check_pkg(paper::CheckPkgInput {
            folder: folder_path.clone(),
        });

        let wallpaper_info = WallpaperInfo {
            wallpaper_id: dir_name,
            title,
            wallpaper_type,
            preview_path,
            has_pkg: pkg_result.has_pkg,
            pkg_files: pkg_result.pkg_files,
            folder_path,
        };

        // 更新统计
        stats.total_count += 1;
        if wallpaper_info.has_pkg {
            stats.pkg_count += 1;
        } else {
            stats.raw_count += 1;
        }

        wallpapers.push(wallpaper_info);
    }

    ScanWallpapersOutput {
        success: true,
        wallpapers,
        stats,
        error: None,
    }
}

/// 复制壁纸到目标目录
/// 
/// 可以选择复制全部或指定的壁纸
pub fn copy_wallpapers(input: CopyWallpapersInput) -> CopyWallpapersOutput {
    // 先扫描获取壁纸列表
    let scan_result = scan_wallpapers(ScanWallpapersInput {
        workshop_path: input.workshop_path.clone(),
    });

    if !scan_result.success {
        return CopyWallpapersOutput {
            success: false,
            results: vec![],
            stats: CopyStats::default(),
            error: scan_result.error,
        };
    }

    // 筛选要处理的壁纸
    let wallpapers_to_process: Vec<_> = match &input.wallpaper_ids {
        Some(ids) => scan_result.wallpapers.into_iter()
            .filter(|w| ids.contains(&w.wallpaper_id))
            .collect(),
        None => scan_result.wallpapers,
    };

    let mut results = Vec::new();
    let mut stats = CopyStats::default();

    for wallpaper in wallpapers_to_process {
        let process_result = paper::process_folder(paper::ProcessFolderInput {
            folder: wallpaper.folder_path.clone(),
            raw_output: input.raw_output_path.clone(),
            pkg_temp_output: input.pkg_temp_path.clone(),
            enable_raw: input.enable_raw,
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
            wallpaper_id: wallpaper.wallpaper_id,
            title: wallpaper.title,
            result_type,
            pkg_files: process_result.pkg_files,
        });
    }

    CopyWallpapersOutput {
        success: true,
        results,
        stats,
        error: None,
    }
}

/// 获取单个壁纸详情
pub fn get_wallpaper_detail(
    workshop_path: &PathBuf,
    wallpaper_id: &str,
) -> Option<WallpaperInfo> {
    let folder_path = workshop_path.join(wallpaper_id);
    
    if !folder_path.exists() || !folder_path.is_dir() {
        return None;
    }

    // 读取元数据
    let meta_result = paper::read_meta(paper::ReadMetaInput {
        folder: folder_path.clone(),
    });

    let (title, wallpaper_type, preview_path) = if meta_result.success {
        let meta = meta_result.meta.unwrap_or_default();
        (
            meta.title,
            meta.wallpaper_type,
            meta.preview.map(|p| folder_path.join(p)),
        )
    } else {
        (None, None, None)
    };

    // 检查 pkg 文件
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
    })
}
