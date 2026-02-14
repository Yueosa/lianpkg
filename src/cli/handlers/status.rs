//! Status 模式处理器

use super::super::args::StatusArgs;
use super::super::output as out;
use lianpkg::api::native::context;
use lianpkg::core::cfg;
use std::path::PathBuf;

/// 执行 status 命令
pub fn run(args: &StatusArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    // 加载配置
    out::debug_api_enter(
        "native",
        "init",
        &format!("config_path={:?}", config_path),
    );
    let config_dir = config_path.map(|p| p.parent().unwrap_or(&p).to_path_buf());
    let ctx = context::init(config_dir).map_err(|e| e.to_string())?;
    out::debug_api_return(&format!(
        "config_path={}",
        ctx.config_path.display()
    ));

    // 清除状态
    if args.clear {
        return clear_state(&ctx, args.yes);
    }

    // 加载状态
    let state = context::load_state_or_default(&ctx.state_path);

    out::title("Status");
    out::path_info("State File", &ctx.state_path);
    println!();

    // 基础统计
    let total = state.processed.len();
    let pkg_count = state
        .processed
        .values()
        .filter(|e| matches!(e.process_type, cfg::ProcessType::Pkg))
        .count();
    let raw_count = state
        .processed
        .values()
        .filter(|e| matches!(e.process_type, cfg::ProcessType::Raw))
        .count();
    let skipped_count = state
        .processed
        .values()
        .filter(|e| matches!(e.process_type, cfg::ProcessType::Skipped))
        .count();

    out::subtitle("Processed Wallpapers");
    out::stat("Total", total);
    out::stat("PKG", pkg_count);
    out::stat("Raw", raw_count);
    out::stat("Skipped", skipped_count);

    if let Some(ref last_run) = state.last_run {
        out::stat("Last Run", last_run);
    }

    println!();

    // 详细列表
    if args.list || args.full {
        if state.processed.is_empty() {
            out::info("No processed wallpapers.");
        } else {
            out::subtitle("Processed List");

            if args.full {
                // 完整详情
                let mut entries: Vec<_> = state.processed.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0));

                for (id, entry) in entries {
                    out::box_start(id);
                    if let Some(ref title) = entry.title {
                        out::box_line("Title", title);
                    }
                    out::box_line("Type", &format!("{:?}", entry.process_type));
                    out::box_line("Processed At", &entry.processed_at);
                    if let Some(ref output) = entry.output_path {
                        out::box_line("Output", output);
                    }
                    out::box_end();
                }
            } else {
                // 简要列表
                out::table_header(&[("ID", 14), ("Title", 28), ("Type", 10), ("Date", 22)]);

                let mut entries: Vec<_> = state.processed.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0));

                for (id, entry) in entries {
                    let title = entry.title.as_deref().unwrap_or("(untitled)");
                    let ptype = format!("{:?}", entry.process_type);
                    let date = &entry.processed_at;

                    out::table_row(&[(id.as_str(), 14), (title, 28), (&ptype, 10), (date, 22)]);
                }
            }
        }
        println!();
    }

    Ok(())
}

/// 清除状态
fn clear_state(ctx: &context::AppContext, yes: bool) -> Result<(), String> {
    if !yes {
        out::warning("This will clear all processing state records.");
        if !out::confirm("Continue?") {
            out::info("Cancelled.");
            return Ok(());
        }
    }

    // 写入空状态
    let empty_state = cfg::StateData::default();
    context::save_state(&ctx.state_path, &empty_state)
        .map_err(|e| e.to_string())?;

    out::success("State cleared.");
    Ok(())
}
