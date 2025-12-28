//! 流水线执行模块
//!
//! 提供完整的 paper → pkg → tex 流水线执行，
//! 支持增量处理、状态跟踪等高级功能。

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::core::cfg;
use super::{
    cfg as native_cfg,
    paper as native_paper,
    pkg as native_pkg,
    tex as native_tex,
};

// ============================================================================
// 结构体定义
// ============================================================================

/// 流水线执行入参
#[derive(Debug, Clone)]
pub struct RunPipelineInput {
    /// 运行时配置
    pub config: native_cfg::RuntimeConfig,
    /// state.json 路径（用于增量处理）
    pub state_path: PathBuf,
    /// 要处理的壁纸 ID 列表，None 表示全部
    pub wallpaper_ids: Option<Vec<String>>,
    /// 参数覆盖（CLI 参数优先级高于配置文件）
    pub overrides: Option<PipelineOverrides>,
    /// 进度回调（可选）
    pub progress_callback: Option<fn(PipelineProgress)>,
}

/// 流水线参数覆盖
/// 
/// CLI 参数可通过此结构覆盖配置文件的设置
#[derive(Debug, Clone, Default)]
pub struct PipelineOverrides {
    /// 覆盖 workshop_path
    pub workshop_path: Option<PathBuf>,
    /// 覆盖 raw_output_path
    pub raw_output_path: Option<PathBuf>,
    /// 覆盖 pkg_temp_path
    pub pkg_temp_path: Option<PathBuf>,
    /// 覆盖 unpacked_output_path
    pub unpacked_output_path: Option<PathBuf>,
    /// 覆盖 converted_output_path
    pub tex_output_path: Option<PathBuf>,
    /// 覆盖 enable_raw_output
    pub enable_raw: Option<bool>,
    /// 覆盖 clean_pkg_temp
    pub clean_pkg_temp: Option<bool>,
    /// 覆盖 clean_unpacked
    pub clean_unpacked: Option<bool>,
    /// 覆盖 incremental
    pub incremental: Option<bool>,
    /// 覆盖 auto_unpack_pkg
    pub auto_unpack_pkg: Option<bool>,
    /// 覆盖 auto_convert_tex
    pub auto_convert_tex: Option<bool>,
}

/// 流水线执行返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunPipelineOutput {
    /// 是否成功
    pub success: bool,
    /// 壁纸处理结果
    pub paper_result: Option<native_paper::CopyWallpapersOutput>,
    /// PKG 解包结果
    pub pkg_result: Option<native_pkg::UnpackAllOutput>,
    /// TEX 转换结果
    pub tex_result: Option<native_tex::ConvertAllOutput>,
    /// 统计信息
    pub stats: PipelineStats,
    /// 错误信息
    pub error: Option<String>,
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
pub struct PipelineProgress {
    /// 当前阶段
    pub stage: PipelineStage,
    /// 当前阶段进度 (0-100)
    pub progress: u8,
    /// 当前处理项目
    pub current_item: Option<String>,
    /// 消息
    pub message: String,
}

/// 流水线阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PipelineStage {
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

/// 简化的流水线执行入参
#[derive(Debug, Clone)]
pub struct QuickRunInput {
    /// 配置目录，None 则使用默认目录
    pub config_dir: Option<PathBuf>,
    /// 是否强制处理所有壁纸（忽略增量）
    pub force_all: bool,
}

/// 简化的流水线执行返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickRunOutput {
    /// 是否成功
    pub success: bool,
    /// 统计信息
    pub stats: PipelineStats,
    /// 错误信息
    pub error: Option<String>,
}

// ============================================================================
// 接口实现
// ============================================================================

/// 执行完整流水线
/// 
/// paper → pkg → tex 完整流程
pub fn run_pipeline(input: RunPipelineInput) -> RunPipelineOutput {
    use std::time::Instant;
    let start_time = Instant::now();

    let mut stats = PipelineStats::default();
    
    // 应用参数覆盖
    let mut config = input.config;
    if let Some(ref overrides) = input.overrides {
        if let Some(ref p) = overrides.workshop_path {
            config.workshop_path = p.clone();
        }
        if let Some(ref p) = overrides.raw_output_path {
            config.raw_output_path = p.clone();
        }
        if let Some(ref p) = overrides.pkg_temp_path {
            config.pkg_temp_path = p.clone();
        }
        if let Some(ref p) = overrides.unpacked_output_path {
            config.unpacked_output_path = p.clone();
        }
        if let Some(ref p) = overrides.tex_output_path {
            config.converted_output_path = Some(p.clone());
        }
        if let Some(enable) = overrides.enable_raw {
            config.enable_raw_output = enable;
        }
        if let Some(clean) = overrides.clean_pkg_temp {
            config.clean_pkg_temp = clean;
        }
        if let Some(clean) = overrides.clean_unpacked {
            config.clean_unpacked = clean;
        }
        if let Some(inc) = overrides.incremental {
            config.pipeline.incremental = inc;
        }
        if let Some(unpack) = overrides.auto_unpack_pkg {
            config.pipeline.auto_unpack_pkg = unpack;
        }
        if let Some(convert) = overrides.auto_convert_tex {
            config.pipeline.auto_convert_tex = convert;
        }
    }

    // 报告进度
    let report_progress = |stage: PipelineStage, progress: u8, item: Option<String>, msg: &str| {
        if let Some(callback) = input.progress_callback {
            callback(PipelineProgress {
                stage,
                progress,
                current_item: item,
                message: msg.to_string(),
            });
        }
    };

    // 阶段1: 加载状态
    report_progress(PipelineStage::Init, 0, None, "Loading state...");
    let mut state = load_or_create_state(&input.state_path);

    // 阶段2: 扫描壁纸
    report_progress(PipelineStage::Scanning, 10, None, "Scanning wallpapers...");
    let scan_result = native_paper::scan_wallpapers(native_paper::ScanWallpapersInput {
        workshop_path: config.workshop_path.clone(),
    });

    if !scan_result.success {
        return RunPipelineOutput {
            success: false,
            paper_result: None,
            pkg_result: None,
            tex_result: None,
            stats,
            error: Some("Failed to scan wallpapers".to_string()),
        };
    }

    // 筛选待处理的壁纸（增量处理）
    let wallpapers_to_process: Vec<String> = if config.pipeline.incremental {
        scan_result.wallpapers.iter()
            .filter(|w| {
                // 检查是否在指定列表中
                let in_list = match &input.wallpaper_ids {
                    Some(ids) => ids.contains(&w.wallpaper_id),
                    None => true,
                };
                // 检查是否已处理
                let not_processed = !native_cfg::is_wallpaper_processed(&state, &w.wallpaper_id);
                in_list && not_processed
            })
            .map(|w| w.wallpaper_id.clone())
            .collect()
    } else {
        match &input.wallpaper_ids {
            Some(ids) => ids.clone(),
            None => scan_result.wallpapers.iter()
                .map(|w| w.wallpaper_id.clone())
                .collect(),
        }
    };

    stats.wallpapers_skipped = scan_result.wallpapers.len() - wallpapers_to_process.len();

    // 阶段3: 复制壁纸
    report_progress(PipelineStage::Copying, 30, None, "Copying wallpapers...");
    let paper_result = native_paper::copy_wallpapers(native_paper::CopyWallpapersInput {
        wallpaper_ids: Some(wallpapers_to_process.clone()),
        workshop_path: config.workshop_path.clone(),
        raw_output_path: config.raw_output_path.clone(),
        pkg_temp_path: config.pkg_temp_path.clone(),
        enable_raw: config.enable_raw_output,
    });

    stats.wallpapers_processed = paper_result.results.len();

    // 更新状态：记录已处理的壁纸
    for result in &paper_result.results {
        let process_type = match result.result_type {
            native_paper::CopyResultType::Raw => cfg::WallpaperProcessType::Raw,
            native_paper::CopyResultType::Pkg => cfg::WallpaperProcessType::Pkg,
            native_paper::CopyResultType::Skipped => cfg::WallpaperProcessType::Skipped,
        };
        
        native_cfg::add_processed_wallpaper(
            &mut state,
            result.wallpaper_id.clone(),
            result.title.clone(),
            process_type,
            None,
        );
    }

    // 阶段4: 解包 PKG（如果启用）
    let pkg_result = if config.pipeline.auto_unpack_pkg && paper_result.stats.pkg_copied > 0 {
        report_progress(PipelineStage::Unpacking, 50, None, "Unpacking PKG files...");
        let result = native_pkg::unpack_all(native_pkg::UnpackAllInput {
            pkg_temp_path: config.pkg_temp_path.clone(),
            unpacked_output_path: config.unpacked_output_path.clone(),
        });
        stats.pkgs_unpacked = result.stats.pkg_success;
        Some(result)
    } else {
        None
    };

    // 阶段5: 转换 TEX（如果启用）
    let tex_result = if config.pipeline.auto_convert_tex {
        if let Some(ref pkg_res) = pkg_result {
            if pkg_res.stats.tex_files > 0 {
                report_progress(PipelineStage::Converting, 70, None, "Converting TEX files...");
                let result = native_tex::convert_all(native_tex::ConvertAllInput {
                    unpacked_path: config.unpacked_output_path.clone(),
                    output_path: config.converted_output_path.clone(),
                });
                stats.texs_converted = result.stats.tex_success;
                Some(result)
            } else {
                None
            }
        } else {
            // 即使没有新的 PKG 解包，也检查是否有待转换的 TEX
            let tex_files = native_pkg::get_tex_files_from_unpacked(&config.unpacked_output_path);
            if !tex_files.is_empty() {
                report_progress(PipelineStage::Converting, 70, None, "Converting TEX files...");
                let result = native_tex::convert_all(native_tex::ConvertAllInput {
                    unpacked_path: config.unpacked_output_path.clone(),
                    output_path: config.converted_output_path.clone(),
                });
                stats.texs_converted = result.stats.tex_success;
                Some(result)
            } else {
                None
            }
        }
    } else {
        None
    };

    // 复制元数据到 tex_converted 目录
    if tex_result.is_some() {
        report_progress(PipelineStage::Cleanup, 85, None, "Copying metadata...");
        copy_metadata_to_tex_converted(&config);
    }

    // 阶段6: 清理
    report_progress(PipelineStage::Cleanup, 90, None, "Cleaning up...");
    
    // 清理 pkg_temp 目录
    if config.clean_pkg_temp {
        let _ = std::fs::remove_dir_all(&config.pkg_temp_path);
    }

    // 清理 unpacked 目录（保留 tex_converted）
    if config.clean_unpacked {
        clean_unpacked_dir(&config.unpacked_output_path);
    }

    // 更新统计并保存状态
    native_cfg::update_statistics(
        &mut state,
        stats.wallpapers_processed as u64,
        stats.pkgs_unpacked as u64,
        stats.texs_converted as u64,
    );

    let _ = native_cfg::save_state(native_cfg::SaveStateInput {
        state_path: input.state_path,
        state,
    });

    stats.elapsed_ms = start_time.elapsed().as_millis() as u64;

    report_progress(PipelineStage::Done, 100, None, "Pipeline completed");

    RunPipelineOutput {
        success: true,
        paper_result: Some(paper_result),
        pkg_result,
        tex_result,
        stats,
        error: None,
    }
}

/// 快速执行流水线
/// 
/// 使用默认配置快速执行完整流水线
pub fn quick_run(input: QuickRunInput) -> QuickRunOutput {
    // 初始化配置
    let init_result = native_cfg::init_config(native_cfg::InitConfigInput {
        config_dir: input.config_dir,
        use_exe_dir: false,  // API 层调用，使用标准路径
    });

    // 加载配置
    let load_result = native_cfg::load_config(native_cfg::LoadConfigInput {
        config_path: init_result.config_path,
    });

    let mut config = match load_result.config {
        Some(cfg) => cfg,
        None => {
            return QuickRunOutput {
                success: false,
                stats: PipelineStats::default(),
                error: Some("Failed to load config".to_string()),
            };
        }
    };

    // 如果强制处理，禁用增量
    if input.force_all {
        config.pipeline.incremental = false;
    }

    // 执行流水线
    let result = run_pipeline(RunPipelineInput {
        config,
        state_path: init_result.state_path,
        wallpaper_ids: None,
        overrides: None,
        progress_callback: None,
    });

    QuickRunOutput {
        success: result.success,
        stats: result.stats,
        error: result.error,
    }
}

/// 仅执行 PKG 解包阶段
pub fn run_pkg_only(
    pkg_temp_path: PathBuf,
    unpacked_output_path: PathBuf,
) -> native_pkg::UnpackAllOutput {
    native_pkg::unpack_all(native_pkg::UnpackAllInput {
        pkg_temp_path,
        unpacked_output_path,
    })
}

/// 仅执行 TEX 转换阶段
pub fn run_tex_only(
    unpacked_path: PathBuf,
    output_path: Option<PathBuf>,
) -> native_tex::ConvertAllOutput {
    native_tex::convert_all(native_tex::ConvertAllInput {
        unpacked_path,
        output_path,
    })
}

// ============================================================================
// 内部工具函数
// ============================================================================

/// 加载或创建状态数据
fn load_or_create_state(state_path: &PathBuf) -> cfg::StateData {
    let load_result = native_cfg::load_state(native_cfg::LoadStateInput {
        state_path: state_path.clone(),
    });

    match load_result.state {
        Some(state) => state,
        None => cfg::StateData::default(),
    }
}

/// 复制元数据文件到 tex_converted 目录
/// 
/// 将 project.json、preview 等文件复制到对应的 tex_converted 目录
/// 
/// 从 Workshop 源目录复制元数据到 tex_converted 目录
/// - 源：workshop_path/壁纸ID/project.json
/// - 目标：Pkg_Unpacked/tex_converted/壁纸ID/project.json
fn copy_metadata_to_tex_converted(config: &native_cfg::RuntimeConfig) {
    use std::fs;
    
    let workshop_path = &config.workshop_path;
    let unpacked_path = &config.unpacked_output_path;
    let tex_converted_base = unpacked_path.join("tex_converted");
    
    // 遍历 tex_converted 目录下的所有壁纸目录
    if let Ok(entries) = fs::read_dir(&tex_converted_base) {
        for entry in entries.flatten() {
            let tex_dest_dir = entry.path();
            if !tex_dest_dir.is_dir() {
                continue;
            }
            
            // 获取壁纸 ID（目录名）
            let wallpaper_id = match tex_dest_dir.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            
            // 源壁纸目录（Steam Workshop）
            let source_dir = workshop_path.join(&wallpaper_id);
            if !source_dir.exists() {
                continue;
            }
            
            // 基础元数据文件（总是尝试复制）
            let base_files = ["project.json", "scene.json"];
            for filename in &base_files {
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
                    // 复制 preview 字段指定的文件
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
}

/// 清理 unpacked 目录（保留 tex_converted）
/// 
/// 目录结构：
/// - 保留：Pkg_Unpacked/tex_converted/
/// - 删除：Pkg_Unpacked/壁纸ID/ (解包中间产物)
fn clean_unpacked_dir(unpacked_path: &PathBuf) {
    use std::fs;
    
    if let Ok(entries) = fs::read_dir(unpacked_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // 保留 tex_converted 目录
            if name == "tex_converted" {
                continue;
            }
            
            // 删除其他目录（壁纸解包中间产物）
            if path.is_dir() {
                let _ = fs::remove_dir_all(&path);
            } else {
                let _ = fs::remove_file(&path);
            }
        }
    }
}
