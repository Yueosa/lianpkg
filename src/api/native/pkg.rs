//! PKG 处理高级接口
//!
//! 封装 core::pkg 的底层操作，提供批量解包等便捷方法。

use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use crate::core::{pkg, path};

// ============================================================================
// 结构体定义
// ============================================================================

/// 批量解包入参
#[derive(Debug, Clone)]
pub struct UnpackAllInput {
    /// Pkg 临时目录（存放待解包的 pkg 文件）
    pub pkg_temp_path: PathBuf,
    /// 解包输出目录
    pub unpacked_output_path: PathBuf,
}

/// 批量解包返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnpackAllOutput {
    /// 是否成功
    pub success: bool,
    /// 解包结果列表
    pub results: Vec<UnpackResult>,
    /// 统计信息
    pub stats: UnpackStats,
    /// 错误信息
    pub error: Option<String>,
}

/// 单个 PKG 解包结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnpackResult {
    /// PKG 文件路径
    pub pkg_path: PathBuf,
    /// PKG 文件名
    pub pkg_name: String,
    /// 场景名称（从 PKG stem 提取）
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

/// 预览 PKG 入参
#[derive(Debug, Clone)]
pub struct PreviewPkgInput {
    /// PKG 文件路径
    pub pkg_path: PathBuf,
}

/// 预览 PKG 返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewPkgOutput {
    /// 是否成功
    pub success: bool,
    /// PKG 信息
    pub pkg_info: Option<PkgPreview>,
    /// 错误信息
    pub error: Option<String>,
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
// 接口实现
// ============================================================================

/// 批量解包 PKG 文件
/// 
/// 扫描 pkg_temp_path 下所有 .pkg 文件并解包到 unpacked_output_path
pub fn unpack_all(input: UnpackAllInput) -> UnpackAllOutput {
    // 确保输出目录存在
    if let Err(e) = path::ensure_dir(&input.unpacked_output_path) {
        return UnpackAllOutput {
            success: false,
            results: vec![],
            stats: UnpackStats::default(),
            error: Some(e),
        };
    }

    // 查找所有 PKG 文件
    let pkg_files = match find_pkg_files(&input.pkg_temp_path) {
        Ok(files) => files,
        Err(e) => {
            return UnpackAllOutput {
                success: false,
                results: vec![],
                stats: UnpackStats::default(),
                error: Some(e),
            };
        }
    };

    let mut results = Vec::new();
    let mut stats = UnpackStats::default();

    for pkg_path in pkg_files {
        stats.pkg_processed += 1;

        let pkg_name = pkg_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let scene_name = path::scene_name_from_pkg_stem(
            pkg_path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
                .as_str()
        );

        let output_dir = input.unpacked_output_path.join(&scene_name);

        // 执行解包
        let unpack_result = pkg::unpack_pkg(pkg::UnpackPkgInput {
            file_path: pkg_path.clone(),
            output_base: output_dir.clone(),
        });

        if unpack_result.success {
            stats.pkg_success += 1;
            
            let files: Vec<UnpackedFile> = unpack_result.extracted_files.iter()
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
                pkg_path,
                pkg_name,
                scene_name,
                output_dir,
                success: true,
                files,
                error: None,
            });
        } else {
            stats.pkg_failed += 1;
            results.push(UnpackResult {
                pkg_path,
                pkg_name,
                scene_name,
                output_dir,
                success: false,
                files: vec![],
                error: unpack_result.error,
            });
        }
    }

    UnpackAllOutput {
        success: stats.pkg_failed == 0,
        results,
        stats,
        error: if stats.pkg_failed > 0 {
            Some(format!("{} PKG files failed to unpack", stats.pkg_failed))
        } else {
            None
        },
    }
}

/// 预览 PKG 文件内容
/// 
/// 不执行解包，只解析显示 PKG 包含的文件列表
pub fn preview_pkg(input: PreviewPkgInput) -> PreviewPkgOutput {
    let parse_result = pkg::parse_pkg(pkg::ParsePkgInput {
        file_path: input.pkg_path,
    });

    if !parse_result.success {
        return PreviewPkgOutput {
            success: false,
            pkg_info: None,
            error: parse_result.error,
        };
    }

    let pkg_info = match parse_result.pkg_info {
        Some(info) => info,
        None => {
            return PreviewPkgOutput {
                success: false,
                pkg_info: None,
                error: Some("PKG info is empty".to_string()),
            };
        }
    };

    let files: Vec<PkgFileEntry> = pkg_info.entries.iter()
        .map(|e| PkgFileEntry {
            name: e.name.clone(),
            size: e.size,
            is_tex: e.name.to_lowercase().ends_with(".tex"),
        })
        .collect();

    let tex_count = files.iter().filter(|f| f.is_tex).count();

    PreviewPkgOutput {
        success: true,
        pkg_info: Some(PkgPreview {
            version: pkg_info.version,
            file_count: pkg_info.file_count,
            files,
            tex_count,
        }),
        error: None,
    }
}

/// 解包单个 PKG 文件
pub fn unpack_single(
    pkg_path: PathBuf,
    output_base: PathBuf,
) -> UnpackResult {
    let pkg_name = pkg_path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let scene_name = path::scene_name_from_pkg_stem(
        pkg_path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
            .as_str()
    );

    let output_dir = output_base.join(&scene_name);

    let unpack_result = pkg::unpack_pkg(pkg::UnpackPkgInput {
        file_path: pkg_path.clone(),
        output_base: output_dir.clone(),
    });

    if unpack_result.success {
        let files: Vec<UnpackedFile> = unpack_result.extracted_files.iter()
            .map(|f| UnpackedFile {
                name: f.entry_name.clone(),
                output_path: f.output_path.clone(),
                size: f.size,
                is_tex: f.entry_name.to_lowercase().ends_with(".tex"),
            })
            .collect();

        UnpackResult {
            pkg_path,
            pkg_name,
            scene_name,
            output_dir,
            success: true,
            files,
            error: None,
        }
    } else {
        UnpackResult {
            pkg_path,
            pkg_name,
            scene_name,
            output_dir,
            success: false,
            files: vec![],
            error: unpack_result.error,
        }
    }
}

/// 获取解包目录下的所有 TEX 文件
pub fn get_tex_files_from_unpacked(unpacked_path: &PathBuf) -> Vec<PathBuf> {
    let mut tex_files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(unpacked_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 递归搜索子目录
                tex_files.extend(get_tex_files_from_unpacked(&path));
            } else if path.extension()
                .map(|e| e.to_string_lossy().to_lowercase() == "tex")
                .unwrap_or(false)
            {
                tex_files.push(path);
            }
        }
    }
    
    tex_files
}

// ============================================================================
// 内部工具函数
// ============================================================================

/// 查找目录下所有 PKG 文件
fn find_pkg_files(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut pkg_files = Vec::new();

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "pkg" {
                    pkg_files.push(path);
                }
            }
        } else if path.is_dir() {
            // 递归搜索子目录
            if let Ok(sub_files) = find_pkg_files(&path) {
                pkg_files.extend(sub_files);
            }
        }
    }

    Ok(pkg_files)
}
