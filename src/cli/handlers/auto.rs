//! Auto 模式处理器（全自动流水线）
//!
//! 直接调用各 API 执行 paper → pkg → tex 流程
//! 支持 -d 调试追踪和 -q 精简输出

use super::super::args::AutoArgs;
use super::super::logger;
use super::super::output as out;
use lianpkg::api::native::{self, paper, pkg, tex};
use lianpkg::core::cfg;
use lianpkg::core::paper as core_paper;
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
        // 先展示 dry-run 信息
        run_dry_run_preview(&config, args, &init_result.state_path)?;

        // 询问用户是否继续
        println!();
        if !out::confirm("Continue with the execution?") {
            out::info("Operation cancelled by user.");
            return Ok(());
        }
        println!();
    }

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
    }

    // ========== 阶段2: 磁盘空间预估 ==========
    let disk_info = estimate_disk_usage(&config, args.quiet)?;

    // ========== 阶段3: 加载状态（增量处理） ==========
    out::debug_api_enter(
        "native",
        "load_state",
        &format!("path={}", init_result.state_path.display()),
    );
    let state_result = native::load_state(native::LoadStateInput {
        state_path: init_result.state_path.clone(),
    });
    let mut state = state_result.state.unwrap_or_default();
    out::debug_api_return(&format!(
        "processed_count={}",
        state.processed_wallpapers.len()
    ));

    // ========== 阶段4: 扫描壁纸 ==========
    if !args.quiet {
        out::subtitle("Executing Pipeline");
        out::progress("Scanning wallpapers...", 0, 100);
    }

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

    // 筛选待处理的壁纸
    let wallpapers_to_process: Vec<String> = filter_wallpapers(
        &scan_result.wallpapers,
        &state,
        args.ids.as_ref(),
        config.pipeline.incremental,
    );

    let wallpapers_skipped = scan_result.wallpapers.len() - wallpapers_to_process.len();
    out::debug_verbose(
        "Filter",
        &format!(
            "to_process={}, skipped={}",
            wallpapers_to_process.len(),
            wallpapers_skipped
        ),
    );

    // ========== 阶段5: 复制壁纸 ==========
    if !args.quiet {
        out::progress("Copying wallpapers...", 20, 100);
    }

    out::debug_api_enter(
        "paper",
        "copy_wallpapers",
        &format!(
            "count={}, enable_raw={}",
            wallpapers_to_process.len(),
            config.enable_raw_output
        ),
    );
    let paper_result = paper::copy_wallpapers(paper::CopyWallpapersInput {
        wallpaper_ids: Some(wallpapers_to_process.clone()),
        workshop_path: config.workshop_path.clone(),
        raw_output_path: config.raw_output_path.clone(),
        pkg_temp_path: config.pkg_temp_path.clone(),
        enable_raw: config.enable_raw_output,
    });
    out::debug_api_return(&format!(
        "raw={}, pkg={}, skipped={}",
        paper_result.stats.raw_copied, paper_result.stats.pkg_copied, paper_result.stats.skipped
    ));

    // 更新状态
    for result in &paper_result.results {
        let process_type = match result.result_type {
            paper::CopyResultType::Raw => cfg::WallpaperProcessType::Raw,
            paper::CopyResultType::Pkg => cfg::WallpaperProcessType::Pkg,
            paper::CopyResultType::Skipped => cfg::WallpaperProcessType::Skipped,
        };
        native::add_processed_wallpaper(
            &mut state,
            result.wallpaper_id.clone(),
            result.title.clone(),
            process_type,
            None,
        );
    }

    // ========== 阶段6: 解包 PKG ==========
    let pkg_result = if config.pipeline.auto_unpack_pkg && paper_result.stats.pkg_copied > 0 {
        if !args.quiet {
            out::progress("Unpacking PKG files...", 40, 100);
        }

        out::debug_api_enter(
            "pkg",
            "unpack_all",
            &format!(
                "input={}, output={}",
                config.pkg_temp_path.display(),
                config.unpacked_output_path.display()
            ),
        );
        let result = pkg::unpack_all(pkg::UnpackAllInput {
            pkg_temp_path: config.pkg_temp_path.clone(),
            unpacked_output_path: config.unpacked_output_path.clone(),
        });
        out::debug_api_return(&format!(
            "success={}, failed={}, files={}, tex={}",
            result.stats.pkg_success,
            result.stats.pkg_failed,
            result.stats.total_files,
            result.stats.tex_files
        ));
        Some(result)
    } else {
        None
    };

    // ========== 阶段7: 转换 TEX ==========
    let tex_result = if config.pipeline.auto_convert_tex {
        let should_convert = pkg_result
            .as_ref()
            .map(|r| r.stats.tex_files > 0)
            .unwrap_or(false);

        if should_convert {
            if !args.quiet {
                out::progress("Converting TEX files...", 60, 100);
            }

            out::debug_api_enter(
                "tex",
                "convert_all",
                &format!(
                    "input={}, output={:?}",
                    config.unpacked_output_path.display(),
                    config.converted_output_path
                ),
            );
            let result = tex::convert_all(tex::ConvertAllInput {
                unpacked_path: config.unpacked_output_path.clone(),
                output_path: config.converted_output_path.clone(),
            });
            out::debug_api_return(&format!(
                "success={}, failed={}, images={}, videos={}",
                result.stats.tex_success,
                result.stats.tex_failed,
                result.stats.image_count,
                result.stats.video_count
            ));
            Some(result)
        } else {
            None
        }
    } else {
        None
    };

    // ========== 阶段7.5: 复制元数据到 tex_converted ==========
    if tex_result.is_some() {
        if !args.quiet {
            out::progress("Copying metadata...", 70, 100);
        }
        out::debug_api_enter(
            "metadata",
            "copy_to_tex_converted",
            &format!(
                "source={}, dest={}",
                config.workshop_path.display(),
                config.unpacked_output_path.display()
            ),
        );
        copy_metadata_to_tex_converted(&config);
        out::debug_api_return("done");
    }

    // ========== 阶段8: 清理 ==========
    if config.clean_pkg_temp {
        if !args.quiet {
            out::progress("Cleaning PKG temp...", 80, 100);
        }
        out::debug_api_enter(
            "cleanup",
            "pkg_temp",
            &config.pkg_temp_path.display().to_string(),
        );
        let _ = std::fs::remove_dir_all(&config.pkg_temp_path);
        out::debug_api_return("done");
    }

    if config.clean_unpacked {
        if !args.quiet {
            out::progress("Cleaning unpacked...", 90, 100);
        }
        out::debug_api_enter("cleanup", "unpacked", "keeping tex_converted");
        cleanup_unpacked(&config.unpacked_output_path);
        out::debug_api_return("done");
    }

    // ========== 阶段9: 保存状态 ==========
    out::debug_api_enter(
        "native",
        "save_state",
        &init_result.state_path.display().to_string(),
    );
    let _ = native::save_state(native::SaveStateInput {
        state_path: init_result.state_path,
        state: state.clone(),
    });
    out::debug_api_return("done");

    // ========== 计算耗时 ==========
    let elapsed_secs = start_time.elapsed().as_secs_f64();

    // ========== 清理进度条 ==========
    if !args.quiet {
        out::clear_progress();
        println!();
    }

    // ========== 输出结果 ==========
    if args.quiet {
        // -q 精简输出
        print_quiet_summary(
            &config,
            &paper_result,
            pkg_result.as_ref(),
            tex_result.as_ref(),
            elapsed_secs,
            &disk_info,
        );
    } else {
        // 正常输出
        print_full_summary(
            &paper_result,
            pkg_result.as_ref(),
            tex_result.as_ref(),
            wallpapers_skipped,
            elapsed_secs,
        );
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
    if let Some(ref p) = args.pkg_temp {
        config.pkg_temp_path = p.clone();
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
    if args.no_clean_temp {
        config.clean_pkg_temp = false;
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

/// 筛选待处理的壁纸
fn filter_wallpapers(
    wallpapers: &[paper::WallpaperInfo],
    state: &cfg::StateData,
    ids: Option<&Vec<String>>,
    incremental: bool,
) -> Vec<String> {
    wallpapers
        .iter()
        .filter(|w| {
            // 检查是否在指定列表中
            let in_list = match ids {
                Some(filter_ids) => filter_ids.contains(&w.wallpaper_id),
                None => true,
            };
            // 增量模式检查是否已处理
            let not_processed = if incremental {
                !native::is_wallpaper_processed(state, &w.wallpaper_id)
            } else {
                true
            };
            in_list && not_processed
        })
        .map(|w| w.wallpaper_id.clone())
        .collect()
}

/// 清理 unpacked 目录（保留 tex_converted）
fn cleanup_unpacked(unpacked_path: &PathBuf) {
    if let Ok(entries) = std::fs::read_dir(unpacked_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 遍历壁纸目录
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        let name = sub_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        // 保留 tex_converted 目录
                        if name != "tex_converted" {
                            let _ = if sub_path.is_dir() {
                                std::fs::remove_dir_all(&sub_path)
                            } else {
                                std::fs::remove_file(&sub_path)
                            };
                        }
                    }
                }
            }
        }
    }
}

/// 将 project.json、preview 等元数据文件复制到对应的 tex_converted 目录
///
/// - 源：workshop_path/壁纸ID/project.json
/// - 目标：Pkg_Unpacked/壁纸ID/tex_converted/project.json
fn copy_metadata_to_tex_converted(config: &native::RuntimeConfig) {
    use std::fs;

    let workshop_path = &config.workshop_path;
    let unpacked_path = &config.unpacked_output_path;

    // 遍历 Pkg_Unpacked 目录下的所有壁纸目录
    if let Ok(entries) = fs::read_dir(unpacked_path) {
        for entry in entries.flatten() {
            let wallpaper_dir = entry.path();
            if !wallpaper_dir.is_dir() {
                continue;
            }

            // 获取壁纸 ID（目录名）
            let wallpaper_id = match wallpaper_dir.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // 检查是否有 tex_converted 子目录
            let tex_dest_dir = wallpaper_dir.join("tex_converted");
            if !tex_dest_dir.exists() {
                continue;
            }

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

/// 磁盘信息
struct DiskInfo {
    #[allow(dead_code)]
    pkg_size: u64,
    #[allow(dead_code)]
    raw_size: u64,
    peak_usage: u64,
}

/// 磁盘空间预估
fn estimate_disk_usage(config: &native::RuntimeConfig, quiet: bool) -> Result<DiskInfo, String> {
    if !quiet {
        out::subtitle_icon("📊", "Disk Usage Estimation");
    }

    let estimate_result = core_paper::estimate(core_paper::EstimateInput {
        search_path: config.workshop_path.clone(),
        enable_raw: config.enable_raw_output,
    });

    let pkg_size = estimate_result.pkg_size;
    let raw_size = estimate_result.raw_size;

    let est_pkg_temp = pkg_size;
    let est_unpacked = (pkg_size as f64 * 1.5) as u64;
    let est_converted = (pkg_size as f64 * 2.0) as u64;

    let peak_usage = est_pkg_temp + est_unpacked + est_converted + raw_size;
    let final_usage = raw_size
        + est_converted
        + if config.clean_unpacked {
            0
        } else {
            est_unpacked
        }
        + if config.clean_pkg_temp {
            0
        } else {
            est_pkg_temp
        };

    if !quiet {
        out::stat_icon("📦", "PKG Files", out::format_size(pkg_size));
        if config.enable_raw_output {
            out::stat_icon("🖼", "Raw Files", out::format_size(raw_size));
        }
        out::stat_icon("📈", "Estimated Peak", out::format_size(peak_usage));
        out::stat_icon("📉", "Estimated Final", out::format_size(final_usage));
    }

    // 检查可用空间
    let check_path = find_existing_parent(&config.unpacked_output_path);
    if let Some(ref p) = check_path {
        if let Ok(available) = fs2::available_space(p) {
            if !quiet {
                out::stat_icon("💾", "Available Space", out::format_size(available));
            }

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

    if !quiet {
        println!();
    }

    Ok(DiskInfo {
        pkg_size,
        raw_size,
        peak_usage,
    })
}

/// 查找存在的父目录
fn find_existing_parent(path: &std::path::Path) -> Option<PathBuf> {
    let mut check_path = path.to_path_buf();
    while !check_path.exists() {
        if let Some(parent) = check_path.parent() {
            check_path = parent.to_path_buf();
        } else {
            return None;
        }
    }
    Some(check_path)
}

/// -q 精简输出
fn print_quiet_summary(
    config: &native::RuntimeConfig,
    paper_result: &paper::CopyWallpapersOutput,
    pkg_result: Option<&pkg::UnpackAllOutput>,
    tex_result: Option<&tex::ConvertAllOutput>,
    elapsed_secs: f64,
    disk_info: &DiskInfo,
) {
    // 格式: LianPkg v0.4.3 | 36 wallpapers | ~5.07 GB peak
    let version = env!("CARGO_PKG_VERSION");
    let wallpaper_count = paper_result.stats.raw_copied + paper_result.stats.pkg_copied;
    println!(
        "LianPkg v{} | {} wallpapers | ~{} peak",
        version,
        wallpaper_count,
        out::format_size(disk_info.peak_usage)
    );

    // 输出路径
    println!("Output: {}", config.unpacked_output_path.display());

    // 格式: Done in 45.2s | 21 PKG → 206 TEX → 196 images
    let pkg_count = pkg_result.map(|r| r.stats.pkg_success).unwrap_or(0);
    let tex_count = tex_result.map(|r| r.stats.tex_success).unwrap_or(0);
    let image_count = tex_result.map(|r| r.stats.image_count).unwrap_or(0);

    println!(
        "Done in {:.1}s | {} PKG → {} TEX → {} images",
        elapsed_secs, pkg_count, tex_count, image_count
    );
}

/// 完整输出
fn print_full_summary(
    paper_result: &paper::CopyWallpapersOutput,
    pkg_result: Option<&pkg::UnpackAllOutput>,
    tex_result: Option<&tex::ConvertAllOutput>,
    wallpapers_skipped: usize,
    elapsed_secs: f64,
) {
    out::title("Summary Report");

    out::subtitle("Wallpaper Extraction");
    out::stat(
        "Processed",
        paper_result.stats.raw_copied + paper_result.stats.pkg_copied,
    );
    out::stat("Skipped (incremental)", wallpapers_skipped);
    out::stat("Raw Copied", paper_result.stats.raw_copied);
    out::stat("PKG Copied", paper_result.stats.pkg_copied);

    if let Some(pkg_res) = pkg_result {
        out::subtitle("PKG Unpack");
        out::stat("PKGs Unpacked", pkg_res.stats.pkg_success);
        out::stat("Files Extracted", pkg_res.stats.total_files);
        out::stat("TEX Files", pkg_res.stats.tex_files);
    }

    if let Some(tex_res) = tex_result {
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
    out::path_info("PKG Temp", &config.pkg_temp_path);
    out::path_info("Unpacked", &config.unpacked_output_path);
    if let Some(ref p) = config.converted_output_path {
        out::path_info("TEX Output", p);
    }

    out::subtitle_icon("⚙", "Options");
    out::option_bool("Enable Raw", config.enable_raw_output);
    out::option_bool("Auto Unpack PKG", config.pipeline.auto_unpack_pkg);
    out::option_bool("Auto Convert TEX", config.pipeline.auto_convert_tex);
    out::option_bool("Incremental", config.pipeline.incremental);
    out::option_bool("Clean PKG Temp", config.clean_pkg_temp);
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
            state_path: (*state_path).to_path_buf(),
        });

        if let Some(state) = state_result.state {
            let processed_count = state.processed_wallpapers.len();
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
        &format!("Copy PKG files to {}", config.pkg_temp_path.display()),
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

    if config.clean_pkg_temp {
        out::step(step, "Clean PKG temp directory");
        step += 1;
    }

    if config.clean_unpacked {
        out::step(step, "Clean unpacked directory (except tex_converted)");
    }

    println!();
    out::success("Dry run completed. Run without --dry-run to execute.");

    Ok(())
}

/// 交互式预览模式（用于执行前确认）
/// 与 dry_run 类似，但不显示最终的 "Dry run completed" 消息
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
            let processed_count = state.processed_wallpapers.len();
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
        &format!("Copy PKG files to {}", config.pkg_temp_path.display()),
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

    if config.clean_pkg_temp {
        out::step(step, "Clean PKG temp directory");
        step += 1;
    }

    if config.clean_unpacked {
        out::step(step, "Clean unpacked directory (except tex_converted)");
    }

    Ok(())
}
