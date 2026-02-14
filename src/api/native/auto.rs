//! 自动批处理流水线
//!
//! 提供完整的 scan → paper → pkg → tex 流水线执行，
//! 支持增量处理、进度回调、磁盘预估等。
//!
//! ## 主要接口
//!
//! - `run_auto`: 执行完整流水线
//! - `estimate_disk_usage`: 预估磁盘使用量
//! - `run_pkg_only`: 仅执行 PKG 解包
//! - `run_tex_only`: 仅执行 TEX 转换

use super::context::{self, AppContext, RuntimeConfig};
use super::pkg as native_pkg;
use super::scan as native_scan;
use super::tex as native_tex;
use super::util;
use crate::core::{cfg, disk, error::CoreResult, paper as core_paper};
use serde::{Deserialize, Serialize};

// ============================================================================
// 回调类型
// ============================================================================

/// 进度回调
pub type ProgressCallback<'a> = &'a dyn Fn(AutoProgress);

// ============================================================================
// 选项与输出类型
// ============================================================================

/// 自动模式选项
///
/// CLI/GUI 负责组装好所有参数，API 层不再做二次覆盖。
pub struct AutoOptions<'a> {
    /// 要处理的壁纸 ID 列表，None 表示全部
    pub wallpaper_ids: Option<Vec<String>>,
    /// 进度回调（可选）
    pub progress: Option<ProgressCallback<'a>>,
}

/// 自动模式输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoOutput {
    /// 壁纸复制结果
    pub copy_output: Option<native_scan::CopyOutput>,
    /// PKG 解包结果
    pub pkg_output: Option<native_pkg::UnpackOutput>,
    /// TEX 转换结果
    pub tex_output: Option<native_tex::ConvertOutput>,
    /// 统计信息
    pub stats: PipelineStats,
}

/// 流水线统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PipelineStats {
    /// 处理的壁纸数
    pub wallpapers_processed: usize,
    /// 跳过的壁纸数（增量处理）
    pub wallpapers_skipped: usize,
    /// 解包的 PKG 数
    pub pkgs_unpacked: usize,
    /// 转换的 TEX 数
    pub texs_converted: usize,
    /// 总耗时（毫秒）
    pub elapsed_ms: u64,
}

/// 流水线进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoProgress {
    /// 当前阶段
    pub stage: AutoStage,
    /// 当前阶段进度 (0-100)
    pub progress: u8,
    /// 当前处理项目
    pub current_item: Option<String>,
    /// 消息
    pub message: String,
}

/// 流水线阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AutoStage {
    /// 初始化
    Init,
    /// 扫描壁纸
    Scanning,
    /// 复制壁纸
    Copying,
    /// 解包 PKG
    Unpacking,
    /// 转换 TEX
    Converting,
    /// 清理
    Cleanup,
    /// 完成
    Done,
}

/// 磁盘预估输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskEstimateOutput {
    /// PKG 文件总大小（字节）
    pub pkg_size: u64,
    /// 原始壁纸总大小（字节）
    pub raw_size: u64,
    /// PKG 壁纸数量
    pub pkg_count: usize,
    /// 原始壁纸数量
    pub raw_count: usize,
    /// 预估解包后大小
    pub estimated_unpacked: u64,
    /// 预估转换后大小
    pub estimated_converted: u64,
    /// 预估峰值使用（字节）
    pub estimated_peak: u64,
    /// 预估最终使用（字节）
    pub estimated_final: u64,
    /// 可用空间（字节）
    pub available_space: Option<u64>,
    /// 空间是否充足
    pub space_sufficient: bool,
}

// ============================================================================
// 流水线执行
// ============================================================================

/// 执行完整自动流水线
///
/// scan → paper(copy) → pkg(unpack) → tex(convert) → cleanup
pub fn run_auto(ctx: &AppContext, opts: AutoOptions) -> CoreResult<AutoOutput> {
    use std::time::Instant;
    let start_time = Instant::now();

    let config = &ctx.config;
    let mut stats = PipelineStats::default();

    let report = |stage: AutoStage, progress: u8, item: Option<String>, msg: &str| {
        if let Some(ref cb) = opts.progress {
            cb(AutoProgress {
                stage,
                progress,
                current_item: item,
                message: msg.to_string(),
            });
        }
    };

    // ========== 阶段 1: 加载状态 ==========
    report(AutoStage::Init, 0, None, "Loading state...");
    let mut state = context::load_state_or_default(&ctx.state_path);

    // ========== 阶段 2: 扫描壁纸 ==========
    report(AutoStage::Scanning, 10, None, "Scanning wallpapers...");
    let scan_result = native_scan::scan_workshop(&config.workshop_path, Some(&state))?;

    // 筛选待处理壁纸
    let wallpapers_to_process = util::filter_wallpapers(
        &scan_result.wallpapers,
        &state,
        opts.wallpaper_ids.as_ref(),
        config.pipeline.incremental,
    );

    stats.wallpapers_skipped = scan_result.wallpapers.len() - wallpapers_to_process.len();

    // 准备待处理的壁纸子集
    let selected_wallpapers: Vec<&native_scan::WallpaperInfo> = scan_result
        .wallpapers
        .iter()
        .filter(|w| wallpapers_to_process.contains(&w.wallpaper_id))
        .collect();
    let selected_owned: Vec<native_scan::WallpaperInfo> =
        selected_wallpapers.into_iter().cloned().collect();

    // ========== 阶段 3: 复制壁纸 ==========
    report(AutoStage::Copying, 30, None, "Copying wallpapers...");
    let copy_output = native_scan::copy_wallpapers(
        &selected_owned,
        &config.raw_output_path,
        config.enable_raw_output,
    )?;

    stats.wallpapers_processed = copy_output.results.len();

    // 更新状态：记录已处理壁纸
    for result in &copy_output.results {
        let process_type = match result.result_type {
            native_scan::CopyResultType::Raw => cfg::ProcessType::Raw,
            native_scan::CopyResultType::Pkg => cfg::ProcessType::Pkg,
            native_scan::CopyResultType::Skipped => cfg::ProcessType::Skipped,
        };
        context::add_processed_wallpaper(
            &mut state,
            result.wallpaper_id.clone(),
            result.title.clone(),
            process_type,
            None,
        );
    }

    // ========== 阶段 4: 解包 PKG ==========
    let pkg_sources: Vec<native_pkg::PkgSource> = copy_output
        .results
        .iter()
        .filter(|r| r.result_type == native_scan::CopyResultType::Pkg)
        .map(|r| native_pkg::PkgSource {
            wallpaper_id: r.wallpaper_id.clone(),
            pkg_paths: r.pkg_files.clone(),
        })
        .collect();

    let pkg_output = if config.pipeline.auto_unpack_pkg && !pkg_sources.is_empty() {
        report(AutoStage::Unpacking, 50, None, "Unpacking PKG files...");
        let result = native_pkg::unpack_all(
            &pkg_sources,
            &config.unpacked_output_path,
        )?;
        stats.pkgs_unpacked = result.stats.pkg_success;
        Some(result)
    } else {
        None
    };

    // ========== 阶段 5: 转换 TEX ==========
    let tex_output = if config.pipeline.auto_convert_tex {
        let has_tex = pkg_output
            .as_ref()
            .map(|r| r.stats.tex_files > 0)
            .unwrap_or(false);

        // 检查是否有 TEX 文件需要转换
        let should_convert = has_tex || !util::find_tex_files(&config.unpacked_output_path).is_empty();

        if should_convert {
            report(AutoStage::Converting, 70, None, "Converting TEX files...");
            let result = native_tex::convert_all(
                &config.unpacked_output_path,
                config.converted_output_path.as_deref(),
            )?;
            stats.texs_converted = result.stats.tex_success;
            Some(result)
        } else {
            None
        }
    } else {
        None
    };

    // ========== 阶段 5.5: 复制元数据 ==========
    if tex_output.is_some() {
        report(AutoStage::Cleanup, 85, None, "Copying metadata...");
        util::copy_metadata_to_tex_converted(config);
    }

    // ========== 阶段 6: 清理 ==========
    report(AutoStage::Cleanup, 90, None, "Cleaning up...");
    if config.clean_unpacked {
        util::clean_unpacked_dir(&config.unpacked_output_path);
    }

    // ========== 阶段 7: 保存状态 ==========
    context::touch_last_run(&mut state);
    let _ = context::save_state(&ctx.state_path, &state);

    stats.elapsed_ms = start_time.elapsed().as_millis() as u64;
    report(AutoStage::Done, 100, None, "Pipeline completed");

    Ok(AutoOutput {
        copy_output: Some(copy_output),
        pkg_output,
        tex_output,
        stats,
    })
}

// ============================================================================
// 磁盘预估
// ============================================================================

/// 预估磁盘使用量
pub fn estimate_disk_usage(config: &RuntimeConfig) -> DiskEstimateOutput {
    let estimate_result = core_paper::estimate(core_paper::EstimateInput {
        search_path: config.workshop_path.clone(),
        enable_raw: config.enable_raw_output,
    });

    let pkg_size = estimate_result.pkg_size;
    let raw_size = estimate_result.raw_size;

    let estimated_unpacked = (pkg_size as f64 * 1.5) as u64;
    let estimated_converted = (pkg_size as f64 * 2.0) as u64;
    let estimated_peak = estimated_unpacked + estimated_converted + raw_size;

    let mut estimated_final = raw_size + estimated_converted;
    if !config.clean_unpacked {
        estimated_final += estimated_unpacked;
    }

    let (available_space, space_sufficient) = match disk::check_space(disk::CheckSpaceInput {
        path: config.unpacked_output_path.clone(),
    }) {
        Ok(space_info) => {
            let sufficient = space_info.available >= estimated_peak;
            (Some(space_info.available), sufficient)
        }
        Err(_) => (None, true),
    };

    DiskEstimateOutput {
        pkg_size,
        raw_size,
        pkg_count: estimate_result.pkg_count,
        raw_count: estimate_result.raw_count,
        estimated_unpacked,
        estimated_converted,
        estimated_peak,
        estimated_final,
        available_space,
        space_sufficient,
    }
}

// ============================================================================
// 专项执行
// ============================================================================

/// 仅执行 PKG 解包
pub fn run_pkg_only(
    pkg_sources: &[native_pkg::PkgSource],
    unpacked_output_path: &std::path::Path,
) -> CoreResult<native_pkg::UnpackOutput> {
    native_pkg::unpack_all(pkg_sources, unpacked_output_path)
}

/// 仅执行 TEX 转换
pub fn run_tex_only(
    unpacked_path: &std::path::Path,
    output_path: Option<&std::path::Path>,
) -> CoreResult<native_tex::ConvertOutput> {
    native_tex::convert_all(unpacked_path, output_path)
}
