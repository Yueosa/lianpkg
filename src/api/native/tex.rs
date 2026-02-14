//! TEX 处理接口
//!
//! 封装 core::tex 的底层操作，提供批量转换、预览等功能。
//! 所有接口返回 `CoreResult<T>`。

use crate::core::{error::CoreResult, tex};
use super::util;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// ============================================================================
// 类型定义
// ============================================================================

/// 批量转换输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOutput {
    /// 转换结果列表
    pub results: Vec<ConvertResult>,
    /// 统计信息
    pub stats: ConvertStats,
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
// 接口
// ============================================================================

/// 批量转换 TEX 文件（多线程）
///
/// 扫描 unpacked_path 下所有 .tex 文件并使用 rayon 并行转换。
pub fn convert_all(
    unpacked_path: &Path,
    output_path: &Path,
) -> CoreResult<ConvertOutput> {
    let tex_files = util::find_tex_files(unpacked_path);

    if tex_files.is_empty() {
        return Ok(ConvertOutput {
            results: vec![],
            stats: ConvertStats::default(),
        });
    }

    let stats = Mutex::new(ConvertStats::default());

    let results: Vec<ConvertResult> = tex_files
        .par_iter()
        .map(|tex_path| {
            let out_path = util::determine_tex_output_path(
                tex_path,
                unpacked_path,
                output_path,
            );

            match tex::convert_tex(tex::ConvertTexInput {
                file_path: tex_path.clone(),
                output_path: out_path.clone(),
            }) {
                Ok(result) => {
                    let info = &result.tex_info;

                    {
                        let mut s = stats.lock().unwrap();
                        s.tex_processed += 1;
                        s.tex_success += 1;
                        if info.is_video {
                            s.video_count += 1;
                        } else {
                            s.image_count += 1;
                        }
                    }

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
                        input_path: tex_path.clone(),
                        output_path: result.converted_file.output_path,
                        success: true,
                        format: Some(result.converted_file.format),
                        tex_info: Some(tex_info),
                        error: None,
                    }
                }
                Err(e) => {
                    {
                        let mut s = stats.lock().unwrap();
                        s.tex_processed += 1;
                        s.tex_failed += 1;
                    }

                    ConvertResult {
                        input_path: tex_path.clone(),
                        output_path: out_path,
                        success: false,
                        format: None,
                        tex_info: None,
                        error: Some(e.to_string()),
                    }
                }
            }
        })
        .collect();

    let stats = stats.into_inner().unwrap();

    Ok(ConvertOutput { results, stats })
}

/// 转换单个 TEX 文件
pub fn convert_single(tex_path: &Path, output_path: &Path) -> CoreResult<ConvertResult> {
    match tex::convert_tex(tex::ConvertTexInput {
        file_path: tex_path.to_path_buf(),
        output_path: output_path.to_path_buf(),
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

            Ok(ConvertResult {
                input_path: tex_path.to_path_buf(),
                output_path: result.converted_file.output_path,
                success: true,
                format: Some(result.converted_file.format),
                tex_info: Some(tex_info),
                error: None,
            })
        }
        Err(e) => Ok(ConvertResult {
            input_path: tex_path.to_path_buf(),
            output_path: output_path.to_path_buf(),
            success: false,
            format: None,
            tex_info: None,
            error: Some(e.to_string()),
        }),
    }
}

/// 预览 TEX 文件信息（不转换）
pub fn preview_tex(tex_path: &Path) -> CoreResult<TexPreview> {
    let result = tex::parse_tex(tex::ParseTexInput {
        file_path: tex_path.to_path_buf(),
    })?;

    let info = result.tex_info;

    Ok(TexPreview {
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
    })
}
