//! TEX 处理高级接口
//!
//! 封装 core::tex 的底层操作，提供批量转换等便捷方法。

use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use crate::core::{tex, path};

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
        let output_path = determine_output_path(
            &tex_path,
            &input.unpacked_path,
            &input.output_path,
        );

        // 执行转换
        let convert_result = tex::convert_tex(tex::ConvertTexInput {
            file_path: tex_path.clone(),
            output_path: output_path.clone(),
        });

        if convert_result.success {
            stats.tex_success += 1;
            
            let tex_info = convert_result.tex_info.as_ref().map(|info| {
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
            });

            let actual_output = convert_result.converted_file
                .as_ref()
                .map(|f| f.output_path.clone())
                .unwrap_or(output_path);

            let format = convert_result.converted_file
                .as_ref()
                .map(|f| f.format.clone());

            results.push(ConvertResult {
                input_path: tex_path,
                output_path: actual_output,
                success: true,
                format,
                tex_info,
                error: None,
            });
        } else {
            stats.tex_failed += 1;
            results.push(ConvertResult {
                input_path: tex_path,
                output_path,
                success: false,
                format: None,
                tex_info: None,
                error: convert_result.error,
            });
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
    let parse_result = tex::parse_tex(tex::ParseTexInput {
        file_path: input.tex_path,
    });

    if !parse_result.success {
        return PreviewTexOutput {
            success: false,
            tex_info: None,
            error: parse_result.error,
        };
    }

    let tex_info = match parse_result.tex_info {
        Some(info) => TexPreview {
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
        },
        None => {
            return PreviewTexOutput {
                success: false,
                tex_info: None,
                error: Some("TEX info is empty".to_string()),
            };
        }
    };

    PreviewTexOutput {
        success: true,
        tex_info: Some(tex_info),
        error: None,
    }
}

/// 转换单个 TEX 文件
pub fn convert_single(
    tex_path: PathBuf,
    output_path: PathBuf,
) -> ConvertResult {
    let convert_result = tex::convert_tex(tex::ConvertTexInput {
        file_path: tex_path.clone(),
        output_path: output_path.clone(),
    });

    if convert_result.success {
        let tex_info = convert_result.tex_info.as_ref().map(|info| TexPreview {
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
        });

        let actual_output = convert_result.converted_file
            .as_ref()
            .map(|f| f.output_path.clone())
            .unwrap_or(output_path);

        let format = convert_result.converted_file
            .as_ref()
            .map(|f| f.format.clone());

        ConvertResult {
            input_path: tex_path,
            output_path: actual_output,
            success: true,
            format,
            tex_info,
            error: None,
        }
    } else {
        ConvertResult {
            input_path: tex_path,
            output_path,
            success: false,
            format: None,
            tex_info: None,
            error: convert_result.error,
        }
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
    tex_path: &PathBuf,
    unpacked_path: &PathBuf,
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
            let output_dir = path::resolve_tex_output_dir(
                None,
                unpacked_path,
                Some(tex_path.as_path()),
                Some(unpacked_path.as_path()),
            );
            let _ = path::ensure_dir(&output_dir);
            output_dir.join(tex_path.file_stem().unwrap_or_default())
        }
    }
}
