//! Wallpaper 模式处理器

use super::super::args::WallpaperArgs;
use super::super::output as out;
use lianpkg::api::native::{self, paper};
use lianpkg::core::path;
use std::path::PathBuf;

/// 执行 wallpaper 命令
pub fn run(args: &WallpaperArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    // 加载配置
    out::debug_api_enter(
        "native",
        "init_config",
        &format!("config_path={:?}", config_path),
    );
    let use_exe_dir = config_path.is_none(); // 无配置路径时 Windows 优先使用 exe 目录
    let init_result = native::init_config(native::InitConfigInput {
        config_dir: config_path.map(|p| p.parent().unwrap_or(&p).to_path_buf()),
        use_exe_dir,
    });
    out::debug_api_return(&format!(
        "config_path={}",
        init_result.config_path.display()
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

    let config = config_result.config.ok_or("Failed to load config")?;

    // 确定路径
    let workshop_path = args
        .path
        .clone()
        .unwrap_or_else(|| config.workshop_path.clone());

    let raw_output = args
        .raw_output
        .clone()
        .unwrap_or_else(|| config.raw_output_path.clone());

    let pkg_temp = args
        .pkg_temp
        .clone()
        .unwrap_or_else(|| config.pkg_temp_path.clone());

    let enable_raw = !args.no_raw && config.enable_raw_output;

    // 预览模式
    if args.preview {
        return run_preview(&workshop_path, args.verbose, args.ids.as_ref());
    }

    // 执行复制
    out::title("Wallpaper Extraction");

    // 调试：显示过滤的 ID
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
    out::path_info("Source", &workshop_path);
    out::path_info("Raw Output", &raw_output);
    out::path_info("PKG Temp", &pkg_temp);
    println!();

    // 确保目录存在
    let _ = path::ensure_dir_compat(&raw_output);
    let _ = path::ensure_dir_compat(&pkg_temp);

    out::debug_api_enter(
        "paper",
        "copy_wallpapers",
        &format!(
            "ids={:?}, workshop={}, enable_raw={}",
            args.ids.as_ref().map(|v| v.len()),
            workshop_path.display(),
            enable_raw
        ),
    );
    let result = paper::copy_wallpapers(paper::CopyWallpapersInput {
        wallpaper_ids: args.ids.clone(),
        workshop_path,
        raw_output_path: raw_output,
        pkg_temp_path: pkg_temp,
        enable_raw,
    });

    if !result.success {
        out::debug_api_error(result.error.as_deref().unwrap_or("Unknown error"));
        return Err(result.error.unwrap_or_else(|| "Unknown error".to_string()));
    }
    out::debug_api_return(&format!(
        "raw={}, pkg={}, skipped={}",
        result.stats.raw_copied, result.stats.pkg_copied, result.stats.skipped
    ));

    // 输出结果
    out::subtitle("Results");
    out::stat("Raw Copied", result.stats.raw_copied);
    out::stat("PKG Copied", result.stats.pkg_copied);
    out::stat("Skipped", result.stats.skipped);
    out::stat("Total PKG Files", result.stats.total_pkg_files);
    println!();

    out::success("Wallpaper extraction completed!");
    Ok(())
}

/// 预览模式
fn run_preview(
    workshop_path: &std::path::Path,
    verbose: bool,
    ids: Option<&Vec<String>>,
) -> Result<(), String> {
    out::title("Wallpaper Preview");
    out::path_info("Workshop", workshop_path);
    println!();

    out::debug_api_enter(
        "paper",
        "scan_wallpapers",
        &format!("path={}", workshop_path.display()),
    );
    let result = paper::scan_wallpapers(paper::ScanWallpapersInput {
        workshop_path: workshop_path.to_path_buf(),
    });

    if !result.success {
        out::debug_api_error(result.error.as_deref().unwrap_or("Failed to scan"));
        return Err(result.error.unwrap_or_else(|| "Failed to scan".to_string()));
    }
    out::debug_api_return(&format!(
        "total={}, pkg={}, raw={}",
        result.stats.total_count, result.stats.pkg_count, result.stats.raw_count
    ));

    // 过滤壁纸（如果指定了 ids）
    let wallpapers: Vec<_> = match ids {
        Some(filter_ids) => {
            let filtered: Vec<_> = result
                .wallpapers
                .iter()
                .filter(|w| filter_ids.contains(&w.wallpaper_id))
                .collect();

            // 检查是否有未找到的 ID
            let not_found: Vec<&str> = filter_ids
                .iter()
                .filter(|id| !result.wallpapers.iter().any(|w| &w.wallpaper_id == *id))
                .map(|s| s.as_str())
                .collect();

            if !not_found.is_empty() {
                out::warning(&format!("IDs not found: {}", not_found.join(", ")));
            }

            if filtered.is_empty() {
                return Err(format!(
                    "No wallpapers found matching IDs: {}",
                    filter_ids.join(", ")
                ));
            }

            filtered
        }
        None => result.wallpapers.iter().collect(),
    };

    out::info(&format!(
        "Found {} wallpapers ({} PKG, {} Raw){}",
        result.stats.total_count,
        result.stats.pkg_count,
        result.stats.raw_count,
        if ids.is_some() {
            format!(", showing {}", wallpapers.len())
        } else {
            String::new()
        }
    ));
    println!();

    if verbose {
        // 详细模式：每个壁纸一个 box
        for wp in &wallpapers {
            out::box_start(&wp.wallpaper_id);
            out::box_line("Title", wp.title.as_deref().unwrap_or("(untitled)"));
            out::box_line("Type", wp.wallpaper_type.as_deref().unwrap_or("unknown"));
            out::box_line("PKG", &out::pkg_badge(wp.has_pkg, Some(wp.pkg_files.len())));
            if !wp.pkg_files.is_empty() {
                let pkg_names: Vec<String> = wp
                    .pkg_files
                    .iter()
                    .map(|p| {
                        p.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    })
                    .collect();
                out::box_line("Files", &pkg_names.join(", "));
            }
            out::box_end();
        }
    } else {
        // 简洁模式：表格
        // ID 列不截断，使用完整宽度
        out::table_header(&[("ID", 14), ("Title", 28), ("Type", 8), ("PKG", 15)]);

        for wp in &wallpapers {
            let title = wp.title.as_deref().unwrap_or("(untitled)");
            let wtype = wp.wallpaper_type.as_deref().unwrap_or("-");
            let pkg_info = if wp.has_pkg {
                format!("✓ ({} files)", wp.pkg_files.len())
            } else {
                "✗".to_string()
            };

            out::table_row(&[
                (&wp.wallpaper_id, 14), // ID 完整显示
                (title, 28),
                (wtype, 8),
                (&pkg_info, 15),
            ]);
        }
    }

    println!();
    Ok(())
}
