//! Wallpaper 模式处理器（扫描 + 复制）

use super::super::args::WallpaperArgs;
use super::super::output as out;
use lianpkg::api::native::scan;
use lianpkg::core::path;
use std::path::PathBuf;

/// 执行 wallpaper 命令
pub fn run(args: &WallpaperArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    let ctx = super::init_context(config_path)?;

    // 确定路径
    let workshop_path = args
        .path
        .clone()
        .unwrap_or_else(|| ctx.config.workshop_path.clone());

    let raw_output = args
        .raw_output
        .clone()
        .unwrap_or_else(|| ctx.config.raw_output_path.clone());

    let enable_raw = !args.no_raw && ctx.config.enable_raw_output;

    // 预览模式
    if args.preview {
        return run_preview(&workshop_path, args.verbose, args.ids.as_ref());
    }

    // 执行复制
    out::title("Wallpaper Extraction");

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
    println!();

    let _ = path::ensure_dir_compat(&raw_output);

    out::debug_api_enter(
        "scan",
        "scan_and_copy",
        &format!(
            "ids={:?}, workshop={}, enable_raw={}",
            args.ids.as_ref().map(|v| v.len()),
            workshop_path.display(),
            enable_raw
        ),
    );
    let result = scan::scan_and_copy(
        &workshop_path,
        args.ids.as_deref(),
        &raw_output,
        enable_raw,
    )
    .map_err(|e| e.to_string())?;

    out::debug_api_return(&format!(
        "raw={}, pkg={}, skipped={}",
        result.stats.raw_copied, result.stats.pkg_copied, result.stats.skipped
    ));

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
        "scan",
        "scan_workshop",
        &format!("path={}", workshop_path.display()),
    );
    let result = scan::scan_workshop(workshop_path, None).map_err(|e| e.to_string())?;

    out::debug_api_return(&format!(
        "total={}, pkg={}, raw={}",
        result.stats.total_count, result.stats.pkg_count, result.stats.raw_count
    ));

    let wallpapers: Vec<_> = match ids {
        Some(filter_ids) => {
            let filtered: Vec<_> = result
                .wallpapers
                .iter()
                .filter(|w| filter_ids.contains(&w.wallpaper_id))
                .collect();

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
                (&wp.wallpaper_id, 14),
                (title, 28),
                (wtype, 8),
                (&pkg_info, 15),
            ]);
        }
    }

    println!();
    Ok(())
}
