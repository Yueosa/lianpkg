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
        use_exe_dir: config_path.is_none(),
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
fn show_status(state: &cfg::StateData, state_path: &std::path::Path, full: bool) -> Result<(), String> {
    out::title("LianPkg Status");
    out::path_info("State File", state_path);
    println!();

    // 上次运行时间
    if let Some(ref last_run) = state.last_run {
        let datetime = format_iso_relative(last_run);
        out::stat("Last Run", datetime);
    } else {
        out::stat("Last Run", "Never");
    }

    out::stat("Total Processed", state.processed.len());
    println!();

    // 处理统计（从 HashMap 实时统计）
    out::subtitle("Processing Statistics");
    let raw_count = state.processed.values()
        .filter(|e| e.process_type == cfg::ProcessType::Raw)
        .count();
    let pkg_count = state.processed.values()
        .filter(|e| e.process_type == cfg::ProcessType::Pkg)
        .count();
    let pkg_tex_count = state.processed.values()
        .filter(|e| e.process_type == cfg::ProcessType::PkgTex)
        .count();
    let skipped_count = state.processed.values()
        .filter(|e| e.process_type == cfg::ProcessType::Skipped)
        .count();

    out::stat("Raw Wallpapers", raw_count);
    out::stat("PKG Wallpapers", pkg_count);
    out::stat("PKG+TEX Wallpapers", pkg_tex_count);
    out::stat("Skipped", skipped_count);

    // 详细模式
    if full && !state.processed.is_empty() {
        // 最近处理的壁纸
        out::subtitle("Recent Wallpapers (Last 5)");
        
        let mut recent: Vec<_> = state.processed.iter().collect();
        recent.sort_by(|a, b| b.1.processed_at.cmp(&a.1.processed_at));
        
        for (id, entry) in recent.iter().take(5) {
            let title = entry.title.as_deref().unwrap_or("(untitled)");
            let time = format_iso_relative(&entry.processed_at);
            let type_str = process_type_str(&entry.process_type);
            
            println!("    {} {} [{}] @ {}", id, title, type_str, time);
        }
    }

    println!();
    Ok(())
}

/// 列出所有已处理壁纸
fn list_processed(state: &cfg::StateData) -> Result<(), String> {
    out::title("Processed Wallpapers");
    
    if state.processed.is_empty() {
        out::info("No wallpapers have been processed yet");
        return Ok(());
    }

    out::info(&format!("Total: {} wallpapers", state.processed.len()));
    println!();

    out::table_header(&[
        ("ID", 12),
        ("Title", 25),
        ("Type", 10),
        ("Processed At", 20),
    ]);

    let mut sorted: Vec<_> = state.processed.iter().collect();
    sorted.sort_by(|a, b| b.1.processed_at.cmp(&a.1.processed_at));

    for (id, entry) in sorted {
        let title = entry.title.as_deref().unwrap_or("(untitled)");
        let type_str = process_type_str(&entry.process_type);
        let time = format_iso_relative(&entry.processed_at);

        out::table_row(&[
            (id.as_str(), 12),
            (title, 25),
            (type_str, 10),
            (&time, 20),
        ]);
    }

    println!();
    Ok(())
}

/// 清除状态
fn clear_status(state_path: &std::path::Path, yes: bool) -> Result<(), String> {
    if !yes {
        out::warning("This will clear all processing history");
        if !out::confirm("Are you sure?") {
            return Err("Operation cancelled".to_string());
        }
    }

    // 删除状态文件
    let _ = cfg::delete_state_json(cfg::DeleteStateInput {
        path: state_path.to_path_buf(),
    });

    // 重新创建空状态
    let _ = cfg::create_state_json(cfg::CreateStateInput {
        path: state_path.to_path_buf(),
        content: None,
    });

    out::success("Status cleared");
    Ok(())
}

/// ProcessType → 显示字符串
fn process_type_str(pt: &cfg::ProcessType) -> &'static str {
    match pt {
        cfg::ProcessType::Raw => "Raw",
        cfg::ProcessType::Pkg => "PKG",
        cfg::ProcessType::PkgTex => "PKG+TEX",
        cfg::ProcessType::Skipped => "Skipped",
    }
}

/// 将 ISO 8601 时间字符串格式化为相对时间或完整时间
fn format_iso_relative(iso: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        let now = chrono::Utc::now();
        let dur = now.signed_duration_since(dt);
        let secs = dur.num_seconds();
        if secs >= 0 && secs < 60 {
            return format!("{} seconds ago", secs);
        } else if secs >= 60 && secs < 3600 {
            return format!("{} minutes ago", secs / 60);
        } else if secs >= 3600 && secs < 86400 {
            return format!("{} hours ago", secs / 3600);
        } else if secs >= 86400 && secs < 604800 {
            return format!("{} days ago", secs / 86400);
        }
        // 超过一周，显示完整日期
        return dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    // 无法解析，原样返回
    iso.to_string()
}
