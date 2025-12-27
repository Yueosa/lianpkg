//! Status 模式处理器

use std::path::PathBuf;
use super::super::args::StatusArgs;
use super::super::output as out;
use lianpkg::api::native;
use lianpkg::core::cfg;

/// 执行 status 命令
pub fn run(args: &StatusArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    // 确定配置目录
    let config_dir = config_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf());

    let init_result = native::init_config(native::InitConfigInput {
        config_dir,
    });

    // 清除状态
    if args.clear {
        return clear_status(&init_result.state_path, args.yes);
    }

    // 加载状态
    let state_result = native::load_state(native::LoadStateInput {
        state_path: init_result.state_path.clone(),
    });

    let state = state_result.state.unwrap_or_default();

    // 列出已处理壁纸
    if args.list {
        return list_processed(&state);
    }

    // 显示统计
    show_status(&state, &init_result.state_path, args.full)
}

/// 显示状态统计
fn show_status(state: &cfg::StateData, state_path: &PathBuf, full: bool) -> Result<(), String> {
    out::title("LianPkg Status");
    out::path_info("State File", state_path);
    println!();

    // 上次运行时间
    if let Some(last_run) = state.last_run {
        let datetime = format_timestamp(last_run);
        out::stat("Last Run", datetime);
    } else {
        out::stat("Last Run", "Never");
    }

    out::stat("Total Runs", state.statistics.total_runs);
    println!();

    // 处理统计
    out::subtitle("Processing Statistics");
    out::stat("Wallpapers Processed", state.statistics.total_wallpapers);
    out::stat("PKGs Unpacked", state.statistics.total_pkgs);
    out::stat("TEXs Converted", state.statistics.total_texs);

    // 详细模式
    if full && !state.processed_wallpapers.is_empty() {
        out::subtitle("Wallpaper Breakdown");
        
        let raw_count = state.processed_wallpapers.iter()
            .filter(|w| w.process_type == cfg::WallpaperProcessType::Raw)
            .count();
        let pkg_count = state.processed_wallpapers.iter()
            .filter(|w| w.process_type == cfg::WallpaperProcessType::Pkg)
            .count();
        let pkg_tex_count = state.processed_wallpapers.iter()
            .filter(|w| w.process_type == cfg::WallpaperProcessType::PkgTex)
            .count();
        let skipped_count = state.processed_wallpapers.iter()
            .filter(|w| w.process_type == cfg::WallpaperProcessType::Skipped)
            .count();

        out::stat("Raw Wallpapers", raw_count);
        out::stat("PKG Wallpapers", pkg_count);
        out::stat("PKG+TEX Wallpapers", pkg_tex_count);
        out::stat("Skipped", skipped_count);

        // 最近处理的壁纸
        out::subtitle("Recent Wallpapers (Last 5)");
        
        let mut recent: Vec<_> = state.processed_wallpapers.iter().collect();
        recent.sort_by(|a, b| b.processed_at.cmp(&a.processed_at));
        
        for wp in recent.iter().take(5) {
            let title = wp.title.as_deref().unwrap_or("(untitled)");
            let time = format_timestamp(wp.processed_at);
            let type_str = match wp.process_type {
                cfg::WallpaperProcessType::Raw => "Raw",
                cfg::WallpaperProcessType::Pkg => "PKG",
                cfg::WallpaperProcessType::PkgTex => "PKG+TEX",
                cfg::WallpaperProcessType::Skipped => "Skipped",
            };
            
            println!("    {} {} [{}] @ {}", wp.wallpaper_id, title, type_str, time);
        }
    }

    println!();
    Ok(())
}

/// 列出所有已处理壁纸
fn list_processed(state: &cfg::StateData) -> Result<(), String> {
    out::title("Processed Wallpapers");
    
    if state.processed_wallpapers.is_empty() {
        out::info("No wallpapers have been processed yet");
        return Ok(());
    }

    out::info(&format!("Total: {} wallpapers", state.processed_wallpapers.len()));
    println!();

    out::table_header(&[
        ("ID", 12),
        ("Title", 25),
        ("Type", 10),
        ("Processed At", 20),
    ]);

    let mut sorted: Vec<_> = state.processed_wallpapers.iter().collect();
    sorted.sort_by(|a, b| b.processed_at.cmp(&a.processed_at));

    for wp in sorted {
        let title = wp.title.as_deref().unwrap_or("(untitled)");
        let type_str = match wp.process_type {
            cfg::WallpaperProcessType::Raw => "Raw",
            cfg::WallpaperProcessType::Pkg => "PKG",
            cfg::WallpaperProcessType::PkgTex => "PKG+TEX",
            cfg::WallpaperProcessType::Skipped => "Skipped",
        };
        let time = format_timestamp(wp.processed_at);

        out::table_row(&[
            (&wp.wallpaper_id, 12),
            (title, 25),
            (type_str, 10),
            (&time, 20),
        ]);
    }

    println!();
    Ok(())
}

/// 清除状态
fn clear_status(state_path: &PathBuf, yes: bool) -> Result<(), String> {
    if !yes {
        out::warning("This will clear all processing history");
        if !out::confirm("Are you sure?") {
            return Err("Operation cancelled".to_string());
        }
    }

    // 删除状态文件
    let _ = cfg::delete_state_json(cfg::DeleteStateInput {
        path: state_path.clone(),
    });

    // 重新创建空状态
    let _ = cfg::create_state_json(cfg::CreateStateInput {
        path: state_path.clone(),
        content: None,
    });

    out::success("Status cleared");
    Ok(())
}

/// 格式化时间戳
fn format_timestamp(timestamp: u64) -> String {
    use std::time::{UNIX_EPOCH, Duration};
    
    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp);
    
    // 简单格式化
    if let Ok(elapsed) = std::time::SystemTime::now().duration_since(datetime) {
        let secs = elapsed.as_secs();
        if secs < 60 {
            return format!("{} seconds ago", secs);
        } else if secs < 3600 {
            return format!("{} minutes ago", secs / 60);
        } else if secs < 86400 {
            return format!("{} hours ago", secs / 3600);
        } else if secs < 604800 {
            return format!("{} days ago", secs / 86400);
        }
    }

    // 回退到完整时间戳
    chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| timestamp.to_string())
}
