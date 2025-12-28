//! Auto 模式处理器（全自动流水线）

use std::path::PathBuf;
use super::super::args::AutoArgs;
use super::super::output as out;
use lianpkg::api::native::{self, pipeline, paper};
use lianpkg::core::paper as core_paper;

/// 执行 auto 命令
pub fn run(args: &AutoArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    // 加载配置
    let use_exe_dir = config_path.is_none();  // 双击 exe 时使用 exe 目录
    let init_result = native::init_config(native::InitConfigInput {
        config_dir: config_path.map(|p| p.parent().unwrap_or(&p).to_path_buf()),
        use_exe_dir,
    });

    let config_result = native::load_config(native::LoadConfigInput {
        config_path: init_result.config_path.clone(),
    });

    let config = config_result.config
        .ok_or("Failed to load config")?;

    // 构建参数覆盖（CLI 参数优先级高于配置文件）
    let overrides = pipeline::PipelineOverrides {
        workshop_path: args.search.clone(),
        raw_output_path: args.raw_output.clone(),
        pkg_temp_path: args.pkg_temp.clone(),
        unpacked_output_path: args.unpacked_output.clone(),
        tex_output_path: args.tex_output.clone(),
        enable_raw: if args.no_raw { Some(false) } else { None },
        clean_pkg_temp: if args.no_clean_temp { Some(false) } else { None },
        clean_unpacked: if args.no_clean_unpacked { Some(false) } else { None },
        // 修复：-I 启用增量，无 -I 则禁用增量（覆盖配置文件默认值）
        incremental: Some(args.incremental),
        auto_unpack_pkg: if args.no_pkg { Some(false) } else { None },
        auto_convert_tex: if args.no_tex { Some(false) } else { None },
    };

    // dry-run 模式
    if args.dry_run {
        // dry_run 需要应用覆盖后的配置
        let mut dry_config = config.clone();
        apply_overrides(&mut dry_config, &overrides);
        return run_dry_run(&dry_config, &args, &init_result.state_path);
    }

    // 显示配置（应用覆盖后）
    let mut display_config = config.clone();
    apply_overrides(&mut display_config, &overrides);
    
    out::title("Auto Mode");
    
    // Debug: 显示配置文件路径
    out::debug_verbose("Config", &init_result.config_path.display().to_string());
    out::debug_verbose("State", &init_result.state_path.display().to_string());
    
    // 调试：显示过滤的 ID
    if let Some(ref ids) = args.ids {
        out::info(&format!("Filtering wallpapers: {} IDs specified", ids.len()));
        for id in ids {
            out::info(&format!("  - {}", id));
        }
        println!();
    }
    
    show_config(&display_config);
    println!();

    // 磁盘空间预估
    estimate_disk_usage(&display_config)?;

    // 执行流水线
    out::subtitle("Executing Pipeline");
    
    let result = pipeline::run_pipeline(pipeline::RunPipelineInput {
        config,
        state_path: init_result.state_path,
        wallpaper_ids: args.ids.clone(),
        overrides: Some(overrides),
        progress_callback: Some(progress_callback),
    });


    out::clear_progress();
    println!();

    if !result.success {
        return Err(result.error.unwrap_or_else(|| "Pipeline failed".to_string()));
    }

    // 输出结果
    out::title("Summary Report");
    
    out::subtitle("Wallpaper Extraction");
    out::stat("Processed", result.stats.wallpapers_processed);
    out::stat("Skipped (incremental)", result.stats.wallpapers_skipped);

    if let Some(ref paper_res) = result.paper_result {
        out::stat("Raw Copied", paper_res.stats.raw_copied);
        out::stat("PKG Copied", paper_res.stats.pkg_copied);
    }

    if let Some(ref pkg_res) = result.pkg_result {
        out::subtitle("PKG Unpack");
        out::stat("PKGs Unpacked", pkg_res.stats.pkg_success);
        out::stat("Files Extracted", pkg_res.stats.total_files);
        out::stat("TEX Files", pkg_res.stats.tex_files);
    }

    if let Some(ref tex_res) = result.tex_result {
        out::subtitle("TEX Conversion");
        out::stat("TEXs Converted", tex_res.stats.tex_success);
        out::stat("Images", tex_res.stats.image_count);
        out::stat("Videos", tex_res.stats.video_count);
    }

    out::subtitle("Performance");
    out::stat("Total Time", format!("{:.2}s", result.stats.elapsed_ms as f64 / 1000.0));

    println!();
    out::success("Auto mode completed successfully!");
    
    Ok(())
}

/// dry-run 模式：只显示计划，不执行
fn run_dry_run(
    config: &native::RuntimeConfig,
    args: &AutoArgs,
    state_path: &PathBuf,
) -> Result<(), String> {
    out::title("Auto Mode (Dry Run)");
    out::warning("This is a dry run - no actual operations will be performed");
    println!();

    show_config(config);
    println!();

    // 扫描壁纸
    out::subtitle("Wallpaper Scan");
    let scan_result = paper::scan_wallpapers(paper::ScanWallpapersInput {
        workshop_path: config.workshop_path.clone(),
    });

    if !scan_result.success {
        return Err("Failed to scan wallpapers".to_string());
    }

    out::stat("Total Wallpapers", scan_result.stats.total_count);
    out::stat("PKG Wallpapers", scan_result.stats.pkg_count);
    out::stat("Raw Wallpapers", scan_result.stats.raw_count);

    // 增量处理统计
    if args.incremental {
        let state_result = native::load_state(native::LoadStateInput {
            state_path: state_path.clone(),
        });

        if let Some(state) = state_result.state {
            let processed_count = state.processed_wallpapers.len();
            let to_process = scan_result.wallpapers.iter()
                .filter(|w| !native::is_wallpaper_processed(&state, &w.wallpaper_id))
                .count();
            
            out::stat("Already Processed", processed_count);
            out::stat("To Be Processed", to_process);
        }
    }

    // 指定 ID 处理
    if let Some(ref ids) = args.ids {
        out::subtitle("Selected Wallpapers");
        for id in ids {
            let found = scan_result.wallpapers.iter().any(|w| &w.wallpaper_id == id);
            if found {
                out::info(&format!("✓ {} found", id));
            } else {
                out::warning(&format!("✗ {} not found", id));
            }
        }
    }

    // 磁盘预估
    estimate_disk_usage(config)?;

    // 执行计划
    out::subtitle("Execution Plan");
    
    let mut step = 1;
    
    if config.enable_raw_output {
        out::info(&format!("{}. Copy raw wallpapers to {}", step, config.raw_output_path.display()));
        step += 1;
    }
    
    out::info(&format!("{}. Copy PKG files to {}", step, config.pkg_temp_path.display()));
    step += 1;

    if config.pipeline.auto_unpack_pkg {
        out::info(&format!("{}. Unpack PKG files to {}", step, config.unpacked_output_path.display()));
        step += 1;
    }

    if config.pipeline.auto_convert_tex {
        let tex_out = config.converted_output_path.as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("{}/*/tex_converted", config.unpacked_output_path.display()));
        out::info(&format!("{}. Convert TEX files to {}", step, tex_out));
        step += 1;
    }

    if config.clean_pkg_temp {
        out::info(&format!("{}. Clean PKG temp directory", step));
        step += 1;
    }

    if config.clean_unpacked {
        out::info(&format!("{}. Clean unpacked directory (except tex_converted)", step));
    }

    println!();
    out::success("Dry run completed. Run without --dry-run to execute.");
    
    Ok(())
}

/// 显示配置信息
fn show_config(config: &native::RuntimeConfig) {
    out::subtitle("Paths");
    out::path_info("Workshop", &config.workshop_path);
    out::path_info("Raw Output", &config.raw_output_path);
    out::path_info("PKG Temp", &config.pkg_temp_path);
    out::path_info("Unpacked", &config.unpacked_output_path);
    if let Some(ref p) = config.converted_output_path {
        out::path_info("TEX Output", p);
    }

    out::subtitle("Options");
    out::stat("Enable Raw", config.enable_raw_output);
    out::stat("Auto Unpack PKG", config.pipeline.auto_unpack_pkg);
    out::stat("Auto Convert TEX", config.pipeline.auto_convert_tex);
    out::stat("Incremental", config.pipeline.incremental);
    out::stat("Clean PKG Temp", config.clean_pkg_temp);
    out::stat("Clean Unpacked", config.clean_unpacked);
}

/// 磁盘空间预估
fn estimate_disk_usage(config: &native::RuntimeConfig) -> Result<(), String> {
    out::subtitle("Disk Usage Estimation");

    // 扫描获取大小估算
    let estimate_result = core_paper::estimate(core_paper::EstimateInput {
        search_path: config.workshop_path.clone(),
        enable_raw: config.enable_raw_output,
    });

    let pkg_size = estimate_result.pkg_size;
    let raw_size = estimate_result.raw_size;

    // 估算各阶段占用
    let est_pkg_temp = pkg_size;
    let est_unpacked = (pkg_size as f64 * 1.5) as u64;
    let est_converted = (pkg_size as f64 * 2.0) as u64;

    let peak_usage = est_pkg_temp + est_unpacked + est_converted + raw_size;
    let final_usage = raw_size + est_converted + 
        if config.clean_unpacked { 0 } else { est_unpacked } +
        if config.clean_pkg_temp { 0 } else { est_pkg_temp };

    out::stat("PKG Files", out::format_size(pkg_size));
    if config.enable_raw_output {
        out::stat("Raw Files", out::format_size(raw_size));
    }
    out::stat("Estimated Peak", out::format_size(peak_usage));
    out::stat("Estimated Final", out::format_size(final_usage));

    // 检查可用空间
    let check_path = find_existing_parent(&config.unpacked_output_path);
    if let Some(ref p) = check_path {
        if let Ok(available) = fs2::available_space(p) {
            out::stat("Available Space", out::format_size(available));
            
            if available < peak_usage {
                out::warning("Insufficient disk space!");
                out::warning(&format!(
                    "Required: {}, Available: {}",
                    out::format_size(peak_usage),
                    out::format_size(available)
                ));
                
                if !out::confirm("Continue anyway?") {
                    return Err("Operation cancelled by user".to_string());
                }
            } else {
                out::success("Disk space OK");
            }
        }
    }

    Ok(())
}

/// 查找存在的父目录
fn find_existing_parent(path: &PathBuf) -> Option<PathBuf> {
    let mut check_path = path.clone();
    while !check_path.exists() {
        if let Some(parent) = check_path.parent() {
            check_path = parent.to_path_buf();
        } else {
            return None;
        }
    }
    Some(check_path)
}

/// 进度回调
fn progress_callback(progress: pipeline::PipelineProgress) {
    let stage_str = match progress.stage {
        pipeline::PipelineStage::Init => "Initializing",
        pipeline::PipelineStage::Scanning => "Scanning",
        pipeline::PipelineStage::Copying => "Copying",
        pipeline::PipelineStage::Unpacking => "Unpacking",
        pipeline::PipelineStage::Converting => "Converting",
        pipeline::PipelineStage::Cleanup => "Cleaning up",
        pipeline::PipelineStage::Done => "Done",
    };
    
    let label = match progress.current_item {
        Some(ref item) => format!("{}: {}", stage_str, item),
        None => stage_str.to_string(),
    };
    
    out::progress(&label, progress.progress as usize, 100);
}

/// 应用参数覆盖到配置（用于 dry-run 和显示）
fn apply_overrides(config: &mut native::RuntimeConfig, overrides: &pipeline::PipelineOverrides) {
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
