//! Auto 模式处理器（全自动流水线）
//!
//! 调用 api::pipeline 执行完整的 paper → pkg → tex 流程
//! 支持 -d 调试追踪和 -q 精简输出

use super::super::args::AutoArgs;
use super::super::logger;
use super::super::output as out;
use lianpkg::api::native::{self, paper, pipeline};
use std::path::PathBuf;
use std::time::Instant;

/// 执行 auto 命令
pub fn run(args: &AutoArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    let start_time = Instant::now();

    // 设置 quiet 模式（仅 auto 支持）
    logger::set_quiet(args.quiet);

    // ========== 阶段1: 加载配置 ==========
    out::debug_api_enter(
        "native",
        "init_config",
        &format!("config_path={:?}", config_path),
    );
    let use_exe_dir = config_path.is_none();
    let init_result = native::init_config(native::InitConfigInput {
        config_dir: config_path.map(|p| p.parent().unwrap_or(&p).to_path_buf()),
        use_exe_dir,
    });
    out::debug_api_return(&format!(
        "config={}, state={}",
        init_result.config_path.display(),
        init_result.state_path.display()
    ));

    out::debug_api_enter(
        "native",
        "load_config",
        &format!("path={}", init_result.config_path.display()),
    );
    let config_result = native::load_config(native::LoadConfigInput {
        config_path: init_result.config_path.clone(),
    });
    out::debug_api_return(&format!("loaded={}", config_result.config.is_some()));

    let mut config = config_result.config.ok_or("Failed to load config")?;

    // 应用 CLI 参数覆盖
    apply_cli_overrides(&mut config, args);

    // dry-run 模式（显式指定 --dry-run）
    if args.dry_run {
        return run_dry_run(&config, args, &init_result.state_path);
    }

    // ========== 交互式确认模式 ==========
    // 非 quiet 模式下，先执行 dry-run 展示，让用户确认路径后再执行
    if !args.quiet {
        run_dry_run_preview(&config, args, &init_result.state_path)?;

        println!();
        if !out::confirm("Continue with the execution?") {
            out::info("Operation cancelled by user.");
            return Ok(());
        }
        println!();
    }

    // ========== 阶段2: 磁盘空间预估 ==========
    let disk_info = estimate_disk_usage(&config, args.quiet)?;

    // ========== 显示配置 ==========
    if !args.quiet {
        out::title("Auto Mode");
        out::debug_verbose("Config", &init_result.config_path.display().to_string());
        out::debug_verbose("State", &init_result.state_path.display().to_string());

        if let Some(ref ids) = args.ids {
            out::info(&format!(
                "Filtering wallpapers: {} IDs specified",
                ids.len()
            ));
            for id in ids {
                out::info(&format!("  - {}", id));
            }
            println!();
        }

        show_config(&config);
        println!();
        out::subtitle("Executing Pipeline");
    }

    // ========== 阶段3: 执行流水线 ==========
    // 构建参数覆盖
    let overrides = build_pipeline_overrides(args);

    // 定义进度回调
    let progress_callback = |progress: pipeline::PipelineProgress| {
        if !logger::is_quiet() {
            out::progress(&progress.message, progress.progress.into(), 100);
        }
    };

    // 定义 debug 日志回调
    let debug_callback = |event: pipeline::DebugLogEvent| {
        render_debug_event(&event);
    };

    // 调用 pipeline API
    let result = pipeline::run_pipeline(pipeline::RunPipelineInput {
        config: config.clone(),
        state_path: init_result.state_path,
        wallpaper_ids: args.ids.clone(),
        overrides: Some(overrides),
        progress_callback: if args.quiet {
            None
        } else {
            Some(&progress_callback)
        },
        debug_logger: if logger::is_debug() {
            Some(&debug_callback)
        } else {
            None
        },
    });

    // ========== 计算耗时 ==========
    let elapsed_secs = start_time.elapsed().as_secs_f64();

    // ========== 清理进度条 ==========
    if !args.quiet {
        out::clear_progress();
        println!();
    }

    // ========== 检查结果 ==========
    if !result.success {
        return Err(result
            .error
            .unwrap_or_else(|| "Pipeline failed".to_string()));
    }

    // ========== 输出结果 ==========
    if args.quiet {
        print_quiet_summary(&config, &result, elapsed_secs, &disk_info);
    } else {
        print_full_summary(&result, elapsed_secs);
    }

    // 重置 quiet 模式
    logger::set_quiet(false);

    Ok(())
}

/// 应用 CLI 参数覆盖到配置
fn apply_cli_overrides(config: &mut native::RuntimeConfig, args: &AutoArgs) {
    if let Some(ref p) = args.search {
        config.workshop_path = p.clone();
    }
    if let Some(ref p) = args.raw_output {
        config.raw_output_path = p.clone();
    }
    if let Some(ref p) = args.unpacked_output {
        config.unpacked_output_path = p.clone();
    }
    if let Some(ref p) = args.tex_output {
        config.converted_output_path = Some(p.clone());
    }
    if args.no_raw {
        config.enable_raw_output = false;
    }
    if args.no_clean_unpacked {
        config.clean_unpacked = false;
    }
    // -I 启用增量，无 -I 则禁用
    config.pipeline.incremental = args.incremental;
    if args.no_tex {
        config.pipeline.auto_convert_tex = false;
    }
}

/// 构建 pipeline 参数覆盖
fn build_pipeline_overrides(args: &AutoArgs) -> pipeline::PipelineOverrides {
    pipeline::PipelineOverrides {
        workshop_path: args.search.clone(),
        raw_output_path: args.raw_output.clone(),
        unpacked_output_path: args.unpacked_output.clone(),
        tex_output_path: args.tex_output.clone(),
        enable_raw: if args.no_raw { Some(false) } else { None },
        clean_unpacked: if args.no_clean_unpacked {
            Some(false)
        } else {
            None
        },
        incremental: Some(args.incremental),
        auto_convert_tex: if args.no_tex { Some(false) } else { None },
    }
}

/// 渲染 debug 日志事件
fn render_debug_event(event: &pipeline::DebugLogEvent) {
    match event.event_type {
        pipeline::DebugLogType::Enter => {
            out::debug_api_enter(&event.module, &event.function, &event.details);
        }
        pipeline::DebugLogType::Return => {
            out::debug_api_return(&event.details);
        }
        pipeline::DebugLogType::Error => {
            out::debug_api_error(&event.details);
        }
    }
}

/// 磁盘预估信息
struct DiskEstimate {
    estimated_peak: u64,
}

/// 磁盘空间预估（使用 pipeline API）
fn estimate_disk_usage(
    config: &native::RuntimeConfig,
    quiet: bool,
) -> Result<DiskEstimate, String> {
    if !quiet {
        out::subtitle_icon("📊", "Disk Usage Estimation");
    }

    // 调用 pipeline API 进行磁盘预估
    let estimate = pipeline::estimate_disk_usage(pipeline::EstimateDiskInput {
        config: config.clone(),
    });

    if !quiet {
        out::stat_icon("📦", "PKG Files", out::format_size(estimate.pkg_size));
        if config.enable_raw_output {
            out::stat_icon("🖼", "Raw Files", out::format_size(estimate.raw_size));
        }
        out::stat_icon(
            "📈",
            "Estimated Peak",
            out::format_size(estimate.estimated_peak),
        );
        out::stat_icon(
            "📉",
            "Estimated Final",
            out::format_size(estimate.estimated_final),
        );

        if let Some(available) = estimate.available_space {
            out::stat_icon("💾", "Available Space", out::format_size(available));

            if !estimate.space_sufficient {
                out::warning("Insufficient disk space!");
                out::warning(&format!(
                    "Required: {}, Available: {}",
                    out::format_size(estimate.estimated_peak),
                    out::format_size(available)
                ));

                if !out::confirm("Continue anyway?") {
                    return Err("Operation cancelled by user".to_string());
                }
            } else {
                out::success("Disk space OK");
            }
        }
        println!();
    }

    Ok(DiskEstimate {
        estimated_peak: estimate.estimated_peak,
    })
}

/// -q 精简输出
fn print_quiet_summary(
    config: &native::RuntimeConfig,
    result: &pipeline::RunPipelineOutput,
    elapsed_secs: f64,
    disk_info: &DiskEstimate,
) {
    let version = env!("CARGO_PKG_VERSION");
    let wallpaper_count = result.stats.wallpapers_processed;

    println!(
        "LianPkg v{} | {} wallpapers | ~{} peak",
        version,
        wallpaper_count,
        out::format_size(disk_info.estimated_peak)
    );

    println!("Output: {}", config.unpacked_output_path.display());

    let pkg_count = result.stats.pkgs_unpacked;
    let tex_count = result.stats.texs_converted;
    let image_count = result
        .tex_result
        .as_ref()
        .map(|r| r.stats.image_count)
        .unwrap_or(0);

    println!(
        "Done in {:.1}s | {} PKG → {} TEX → {} images",
        elapsed_secs, pkg_count, tex_count, image_count
    );
}

/// 完整输出
fn print_full_summary(result: &pipeline::RunPipelineOutput, elapsed_secs: f64) {
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
    out::stat("Total Time", format!("{:.2}s", elapsed_secs));

    println!();
    out::success("Auto mode completed successfully!");
}

/// 显示配置信息
fn show_config(config: &native::RuntimeConfig) {
    out::subtitle_icon("📁", "Paths");
    out::path_info("Workshop", &config.workshop_path);
    out::path_info("Raw Output", &config.raw_output_path);
    out::path_info("Unpacked", &config.unpacked_output_path);
    if let Some(ref p) = config.converted_output_path {
        out::path_info("TEX Output", p);
    }

    out::subtitle_icon("⚙", "Options");
    out::option_bool("Enable Raw", config.enable_raw_output);
    out::option_bool("Auto Unpack PKG", config.pipeline.auto_unpack_pkg);
    out::option_bool("Auto Convert TEX", config.pipeline.auto_convert_tex);
    out::option_bool("Incremental", config.pipeline.incremental);
    out::option_bool("Clean Unpacked", config.clean_unpacked);
}

/// dry-run 模式
fn run_dry_run(
    config: &native::RuntimeConfig,
    args: &AutoArgs,
    state_path: &std::path::Path,
) -> Result<(), String> {
    out::title("Auto Mode (Dry Run)");
    out::warning("This is a dry run - no actual operations will be performed");
    println!();

    show_config(config);
    println!();

    // 扫描壁纸
    out::subtitle_icon("🔍", "Wallpaper Scan");
    out::debug_api_enter(
        "paper",
        "scan_wallpapers",
        &format!("path={}", config.workshop_path.display()),
    );
    let scan_result = paper::scan_wallpapers(paper::ScanWallpapersInput {
        workshop_path: config.workshop_path.clone(),
    });

    if !scan_result.success {
        out::debug_api_error("Failed to scan wallpapers");
        return Err("Failed to scan wallpapers".to_string());
    }
    out::debug_api_return(&format!(
        "total={}, pkg={}, raw={}",
        scan_result.stats.total_count, scan_result.stats.pkg_count, scan_result.stats.raw_count
    ));

    out::stat_icon("📦", "Total Wallpapers", scan_result.stats.total_count);
    out::stat_icon("📁", "PKG Wallpapers", scan_result.stats.pkg_count);
    out::stat_icon("🖼", "Raw Wallpapers", scan_result.stats.raw_count);

    // 增量处理统计
    if args.incremental {
        let state_result = native::load_state(native::LoadStateInput {
            state_path: state_path.to_path_buf(),
        });

        if let Some(state) = state_result.state {
            let processed_count = state.processed.len();
            let to_process = scan_result
                .wallpapers
                .iter()
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
    estimate_disk_usage(config, false)?;

    // 执行计划
    show_execution_plan(config);

    println!();
    out::success("Dry run completed. Run without --dry-run to execute.");

    Ok(())
}

/// 交互式预览模式（用于执行前确认）
fn run_dry_run_preview(
    config: &native::RuntimeConfig,
    args: &AutoArgs,
    state_path: &std::path::Path,
) -> Result<(), String> {
    out::title("Auto Mode Preview");
    out::warning("Please review the configuration before execution");
    println!();

    show_config(config);
    println!();

    // 扫描壁纸
    out::subtitle_icon("🔍", "Wallpaper Scan");
    out::debug_api_enter(
        "paper",
        "scan_wallpapers",
        &format!("path={}", config.workshop_path.display()),
    );
    let scan_result = paper::scan_wallpapers(paper::ScanWallpapersInput {
        workshop_path: config.workshop_path.clone(),
    });

    if !scan_result.success {
        out::debug_api_error("Failed to scan wallpapers");
        return Err("Failed to scan wallpapers".to_string());
    }
    out::debug_api_return(&format!(
        "total={}, pkg={}, raw={}",
        scan_result.stats.total_count, scan_result.stats.pkg_count, scan_result.stats.raw_count
    ));

    out::stat_icon("📦", "Total Wallpapers", scan_result.stats.total_count);
    out::stat_icon("📁", "PKG Wallpapers", scan_result.stats.pkg_count);
    out::stat_icon("🖼", "Raw Wallpapers", scan_result.stats.raw_count);

    // 增量处理统计
    if args.incremental {
        let state_result = native::load_state(native::LoadStateInput {
            state_path: state_path.to_path_buf(),
        });

        if let Some(state) = state_result.state {
            let processed_count = state.processed.len();
            let to_process = scan_result
                .wallpapers
                .iter()
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

    // 磁盘预估（使用 pipeline API）
    estimate_disk_usage(config, false)?;

    // 执行计划
    show_execution_plan(config);

    Ok(())
}

/// 显示执行计划
fn show_execution_plan(config: &native::RuntimeConfig) {
    out::subtitle_icon("📝", "Execution Plan");

    let mut step = 1;

    if config.enable_raw_output {
        out::step(
            step,
            &format!(
                "Copy raw wallpapers to {}",
                config.raw_output_path.display()
            ),
        );
        step += 1;
    }

    out::step(
        step,
        "Scan PKG files from Workshop (no temp copy)",
    );
    step += 1;

    if config.pipeline.auto_unpack_pkg {
        out::step(
            step,
            &format!(
                "Unpack PKG files to {}",
                config.unpacked_output_path.display()
            ),
        );
        step += 1;
    }

    if config.pipeline.auto_convert_tex {
        let tex_out = config
            .converted_output_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| {
                format!("{}/*/tex_converted", config.unpacked_output_path.display())
            });
        out::step(step, &format!("Convert TEX files to {}", tex_out));
        step += 1;
    }

    if config.clean_unpacked {
        out::step(step, "Clean unpacked directory (except tex_converted)");
    }
}
