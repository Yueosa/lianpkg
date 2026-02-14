//! Auto 模式处理器（全自动流水线）

use super::super::args::AutoArgs;
use super::super::output as out;
use lianpkg::api::native::{auto, context};
use std::path::PathBuf;

/// 执行 auto 命令
pub fn run(args: &AutoArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    // 加载配置
    out::debug_api_enter(
        "native",
        "init",
        &format!("config_path={:?}", config_path),
    );
    let config_dir = config_path.map(|p| p.parent().unwrap_or(&p).to_path_buf());
    let mut ctx = context::init(config_dir).map_err(|e| e.to_string())?;
    out::debug_api_return(&format!(
        "config_path={}",
        ctx.config_path.display()
    ));

    // 应用 CLI 覆盖参数到配置
    apply_overrides(&mut ctx.config, args);

    // dry-run 模式：显示配置 + 磁盘估算
    if args.dry_run {
        return run_dry_run(&ctx, args);
    }

    // 精简模式
    let quiet = args.quiet;

    if !quiet {
        out::title("Auto Pipeline");
        print_config(&ctx.config, args);
        println!();
    }

    // 磁盘估算
    if !quiet {
        let estimate = auto::estimate_disk_usage(&ctx.config);
        out::subtitle("Disk Estimate");
        out::stat("PKG Size", out::format_size(estimate.pkg_size));
        out::stat("Raw Size", out::format_size(estimate.raw_size));
        out::stat("Peak Usage", out::format_size(estimate.estimated_peak));
        out::stat("Final Usage", out::format_size(estimate.estimated_final));
        if let Some(avail) = estimate.available_space {
            out::stat("Available", out::format_size(avail));
        }
        if !estimate.space_sufficient {
            out::warning("Disk space may be insufficient!");
        }
        println!();
    }

    // 构建选项
    let progress_cb = if quiet {
        None
    } else {
        Some(&progress_handler as &dyn Fn(auto::AutoProgress))
    };

    let opts = auto::AutoOptions {
        wallpaper_ids: args.ids.clone(),
        progress: progress_cb,
    };

    // 执行流水线
    out::debug_api_enter("auto", "run_auto", "full pipeline");
    let result = auto::run_auto(&ctx, opts).map_err(|e| e.to_string())?;
    out::debug_api_return(&format!(
        "wallpapers={}, pkgs={}, texs={}, elapsed={}ms",
        result.stats.wallpapers_processed,
        result.stats.pkgs_unpacked,
        result.stats.texs_converted,
        result.stats.elapsed_ms,
    ));

    // 清除进度行
    if !quiet {
        out::clear_progress();
    }

    // 输出结果
    if quiet {
        out::quiet_result(
            result.stats.elapsed_ms as f64 / 1000.0,
            result.stats.pkgs_unpacked,
            result.stats.texs_converted,
        );
    } else {
        print_results(&result);
    }

    Ok(())
}

/// 将 CLI 参数覆盖到运行时配置中
fn apply_overrides(config: &mut context::RuntimeConfig, args: &AutoArgs) {
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
    if args.no_tex {
        config.pipeline.auto_convert_tex = false;
    }
    if args.no_clean_unpacked {
        config.clean_unpacked = false;
    }
    if args.incremental {
        config.pipeline.incremental = true;
    }
}

/// dry-run 模式
fn run_dry_run(ctx: &context::AppContext, args: &AutoArgs) -> Result<(), String> {
    out::title("Auto Pipeline (Dry Run)");
    print_config(&ctx.config, args);
    println!();

    let estimate = auto::estimate_disk_usage(&ctx.config);

    out::subtitle("Disk Estimate");
    out::stat("PKG Wallpapers", estimate.pkg_count);
    out::stat("Raw Wallpapers", estimate.raw_count);
    out::stat("PKG Total Size", out::format_size(estimate.pkg_size));
    out::stat("Raw Total Size", out::format_size(estimate.raw_size));
    println!();
    out::stat("Est. Unpacked", out::format_size(estimate.estimated_unpacked));
    out::stat("Est. Converted", out::format_size(estimate.estimated_converted));
    out::stat("Est. Peak Usage", out::format_size(estimate.estimated_peak));
    out::stat("Est. Final", out::format_size(estimate.estimated_final));
    println!();

    if let Some(avail) = estimate.available_space {
        out::stat("Available Space", out::format_size(avail));
        if estimate.space_sufficient {
            out::success("Disk space is sufficient");
        } else {
            out::warning("Disk space may be insufficient!");
        }
    } else {
        out::warning("Unable to determine available disk space");
    }

    println!();
    out::info("Dry run completed. No changes were made.");
    Ok(())
}

/// 打印配置摘要
fn print_config(config: &context::RuntimeConfig, args: &AutoArgs) {
    out::subtitle("Configuration");
    out::path_info("Workshop", &config.workshop_path);
    out::path_info("Raw Output", &config.raw_output_path);
    out::path_info("Unpacked Output", &config.unpacked_output_path);
    if let Some(ref p) = config.converted_output_path {
        out::path_info("TEX Output", p);
    }
    println!();

    out::option_bool("Raw Output", config.enable_raw_output);
    out::option_bool("Auto Unpack PKG", config.pipeline.auto_unpack_pkg);
    out::option_bool("Auto Convert TEX", config.pipeline.auto_convert_tex);
    out::option_bool("Incremental", config.pipeline.incremental);
    out::option_bool("Clean Unpacked", config.clean_unpacked);

    if let Some(ref ids) = args.ids {
        out::info(&format!("Filter: {} wallpaper IDs", ids.len()));
    }
}

/// 进度回调
fn progress_handler(progress: auto::AutoProgress) {
    let stage_name = match progress.stage {
        auto::AutoStage::Init => "Init",
        auto::AutoStage::Scanning => "Scan",
        auto::AutoStage::Copying => "Copy",
        auto::AutoStage::Unpacking => "Unpack",
        auto::AutoStage::Converting => "Convert",
        auto::AutoStage::Cleanup => "Cleanup",
        auto::AutoStage::Done => "Done",
    };

    out::progress(
        &format!("[{}] {}", stage_name, progress.message),
        progress.progress as usize,
        100,
    );
}

/// 输出结果
fn print_results(result: &auto::AutoOutput) {
    println!();
    out::subtitle("Pipeline Results");

    // 壁纸复制结果
    if let Some(ref copy) = result.copy_output {
        out::step(1, "Wallpaper Copy");
        out::stat("  Raw Copied", copy.stats.raw_copied);
        out::stat("  PKG Copied", copy.stats.pkg_copied);
        out::stat("  Skipped", copy.stats.skipped);
    }

    // PKG 解包结果
    if let Some(ref pkg) = result.pkg_output {
        out::step(2, "PKG Unpack");
        out::stat("  PKGs Processed", pkg.stats.pkg_processed);
        out::stat("  PKGs Success", pkg.stats.pkg_success);
        out::stat("  PKGs Failed", pkg.stats.pkg_failed);
        out::stat("  Total Files", pkg.stats.total_files);
        out::stat("  TEX Files", pkg.stats.tex_files);
    }

    // TEX 转换结果
    if let Some(ref tex) = result.tex_output {
        out::step(3, "TEX Convert");
        out::stat("  TEX Processed", tex.stats.tex_processed);
        out::stat("  TEX Success", tex.stats.tex_success);
        out::stat("  TEX Failed", tex.stats.tex_failed);
        out::stat("  Images", tex.stats.image_count);
        out::stat("  Videos", tex.stats.video_count);
    }

    // 总结
    println!();
    out::subtitle("Summary");
    out::stat("Wallpapers Processed", result.stats.wallpapers_processed);
    out::stat("Wallpapers Skipped", result.stats.wallpapers_skipped);
    out::stat("PKGs Unpacked", result.stats.pkgs_unpacked);
    out::stat("TEXs Converted", result.stats.texs_converted);
    out::stat(
        "Total Time",
        format!("{:.1}s", result.stats.elapsed_ms as f64 / 1000.0),
    );
    println!();

    out::success("Auto pipeline completed!");
}
