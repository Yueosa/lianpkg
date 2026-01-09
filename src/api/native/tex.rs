//! TEX 处理高级接口
//!
//! 封装 core::tex 的底层操作，提供批量转换等便捷方法。

use crate::core::{path, tex};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// 结构体定义
// ============================================================================

/// 批量转换入参
#[derive(Debug, Clone)]
pub struct ConvertAllInput {
    /// 解包输出目录（从此目录搜索 TEX 文件）
    pub unpacked_path: PathBuf,
    /// 转换输出目录，None 则输出到解包目录下的 tex_converted 子目录
    pub output_path: Option<PathBuf>,
}

/// 批量转换返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertAllOutput {
    /// 是否成功（全部成功才为 true）
    pub success: bool,
    /// 转换结果列表
    pub results: Vec<ConvertResult>,
    /// 统计信息
    pub stats: ConvertStats,
    /// 错误信息
    pub error: Option<String>,
}

/// 单个 TEX 转换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertResult {
    /// 输入 TEX 文件路径
    pub input_path: PathBuf,
    /// 输出文件路径
    pub output_path: PathBuf,
    /// 是否成功
    pub success: bool,
    /// 输出格式
    pub format: Option<String>,
    /// TEX 信息
    pub tex_info: Option<TexPreview>,
    /// 错误信息
    pub error: Option<String>,
}

/// 转换统计
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ConvertStats {
    /// 处理的 TEX 文件数
    pub tex_processed: usize,
    /// 成功转换数
    pub tex_success: usize,
    /// 失败数
    pub tex_failed: usize,
    /// 跳过数（非 TEX 格式等）
    pub tex_skipped: usize,
    /// 图片输出数
    pub image_count: usize,
    /// 视频输出数
    pub video_count: usize,
}

/// 预览 TEX 入参
#[derive(Debug, Clone)]
pub struct PreviewTexInput {
    /// TEX 文件路径
    pub tex_path: PathBuf,
}

/// 预览 TEX 返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewTexOutput {
    /// 是否成功
    pub success: bool,
    /// TEX 信息
    pub tex_info: Option<TexPreview>,
    /// 错误信息
    pub error: Option<String>,
}

/// TEX 预览信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TexPreview {
    /// TEX 版本
    pub version: String,
    /// 格式类型
    pub format: String,
    /// 图像宽度
    pub width: u32,
    /// 图像高度
    pub height: u32,
    /// 图像数量
    pub image_count: usize,
    /// Mipmap 数量
    pub mipmap_count: usize,
    /// 是否 LZ4 压缩
    pub is_compressed: bool,
    /// 是否视频
    pub is_video: bool,
    /// 数据大小（字节）
    pub data_size: usize,
    /// 推荐输出格式
    pub recommended_output: String,
}

// ============================================================================
// 接口实现
// ============================================================================

/// 批量转换 TEX 文件
///
/// 扫描 unpacked_path 下所有 .tex 文件并转换
pub fn convert_all(input: ConvertAllInput) -> ConvertAllOutput {
    // 查找所有 TEX 文件
    let tex_files = find_tex_files(&input.unpacked_path);

    if tex_files.is_empty() {
        return ConvertAllOutput {
            success: true,
            results: vec![],
            stats: ConvertStats::default(),
            error: None,
        };
    }

    let mut results = Vec::new();
    let mut stats = ConvertStats::default();

    for tex_path in tex_files {
        stats.tex_processed += 1;

        // 确定输出路径
        let output_path =
            determine_output_path(&tex_path, &input.unpacked_path, &input.output_path);

        // 执行转换
        let convert_result = tex::convert_tex(tex::ConvertTexInput {
            file_path: tex_path.clone(),
            output_path: output_path.clone(),
        });

        match convert_result {
            Ok(result) => {
                stats.tex_success += 1;

                let tex_info = {
                    let info = &result.tex_info;
                    if info.is_video {
                        stats.video_count += 1;
                    } else {
                        stats.image_count += 1;
                    }

                    TexPreview {
                        version: info.version.clone(),
                        format: info.format.clone(),
                        width: info.width,
                        height: info.height,
                        image_count: info.image_count,
                        mipmap_count: info.mipmap_count,
                        is_compressed: info.is_compressed,
                        is_video: info.is_video,
                        data_size: info.data_size,
                        recommended_output: if info.is_video { "mp4" } else { "png" }.to_string(),
                    }
                };

                results.push(ConvertResult {
                    input_path: tex_path,
                    output_path: result.converted_file.output_path,
                    success: true,
                    format: Some(result.converted_file.format),
                    tex_info: Some(tex_info),
                    error: None,
                });
            }
            Err(e) => {
                stats.tex_failed += 1;
                results.push(ConvertResult {
                    input_path: tex_path,
                    output_path,
                    success: false,
                    format: None,
                    tex_info: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    ConvertAllOutput {
        success: stats.tex_failed == 0,
        results,
        stats,
        error: if stats.tex_failed > 0 {
            Some(format!("{} TEX files failed to convert", stats.tex_failed))
        } else {
            None
        },
    }
}

/// 预览 TEX 文件信息
///
/// 不执行转换，只解析显示 TEX 文件的格式信息
pub fn preview_tex(input: PreviewTexInput) -> PreviewTexOutput {
    match tex::parse_tex(tex::ParseTexInput {
        file_path: input.tex_path,
    }) {
        Ok(result) => {
            let info = result.tex_info;
            PreviewTexOutput {
                success: true,
                tex_info: Some(TexPreview {
                    version: info.version,
                    format: info.format,
                    width: info.width,
                    height: info.height,
                    image_count: info.image_count,
                    mipmap_count: info.mipmap_count,
                    is_compressed: info.is_compressed,
                    is_video: info.is_video,
                    data_size: info.data_size,
                    recommended_output: if info.is_video { "mp4" } else { "png" }.to_string(),
                }),
                error: None,
            }
        }
        Err(e) => PreviewTexOutput {
            success: false,
            tex_info: None,
            error: Some(e.to_string()),
        },
    }
}

/// 转换单个 TEX 文件
pub fn convert_single(tex_path: PathBuf, output_path: PathBuf) -> ConvertResult {
    match tex::convert_tex(tex::ConvertTexInput {
        file_path: tex_path.clone(),
        output_path: output_path.clone(),
    }) {
        Ok(result) => {
            let info = &result.tex_info;
            let tex_info = TexPreview {
                version: info.version.clone(),
                format: info.format.clone(),
                width: info.width,
                height: info.height,
                image_count: info.image_count,
                mipmap_count: info.mipmap_count,
                is_compressed: info.is_compressed,
                is_video: info.is_video,
                data_size: info.data_size,
                recommended_output: if info.is_video { "mp4" } else { "png" }.to_string(),
            };

            ConvertResult {
                input_path: tex_path,
                output_path: result.converted_file.output_path,
                success: true,
                format: Some(result.converted_file.format),
                tex_info: Some(tex_info),
                error: None,
            }
        }
        Err(e) => ConvertResult {
            input_path: tex_path,
            output_path,
            success: false,
            format: None,
            tex_info: None,
            error: Some(e.to_string()),
        },
    }
}

// ============================================================================
// 内部工具函数
// ============================================================================

/// 查找目录下所有 TEX 文件
fn find_tex_files(dir: &PathBuf) -> Vec<PathBuf> {
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
                // 递归搜索子目录
                tex_files.extend(find_tex_files(&path));
            }
        }
    }

    tex_files
}

/// 确定输出路径
fn determine_output_path(
    tex_path: &std::path::Path,
    unpacked_path: &std::path::Path,
    custom_output: &Option<PathBuf>,
) -> PathBuf {
    match custom_output {
        Some(output_base) => {
            // 使用自定义输出目录，保持相对路径结构
            if let Ok(relative) = tex_path.strip_prefix(unpacked_path) {
                output_base.join(relative).with_extension("")
            } else {
                output_base.join(tex_path.file_stem().unwrap_or_default())
            }
        }
        None => {
            // 使用默认的 tex_converted 子目录
            // 需要找到壁纸的根目录（scene_root）
            let scene_root = if let Ok(relative) = tex_path.strip_prefix(unpacked_path) {
                // relative 如: "123456/materials/scene.tex"
                // 取第一级目录: "123456"
                if let Some(first_component) = relative.components().next() {
                    unpacked_path.join(first_component.as_os_str())
                } else {
                    unpacked_path.to_path_buf()
                }
            } else {
                // 无法确定相对路径，使用 tex 文件的父目录
                tex_path.parent().unwrap_or(unpacked_path).to_path_buf()
            };

            let output_dir = path::resolve_tex_output_dir_compat(
                None,
                &scene_root,
                Some(tex_path),
                Some(&scene_root),
            );
            let _ = path::ensure_dir_compat(&output_dir);
            output_dir.join(tex_path.file_stem().unwrap_or_default())
        }
    }
}
