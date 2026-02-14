//! Show 模式处理器 — 查看单个壁纸详情

use super::super::args::ShowArgs;
use super::super::output as out;
use lianpkg::api::native::{pkg, scan};
use std::path::PathBuf;

/// 执行 show 命令
pub fn run(args: &ShowArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    let ctx = super::init_context(config_path)?;

    let workshop_path = &ctx.config.workshop_path;

    // 查找壁纸
    let wallpaper = scan::get_wallpaper_detail(workshop_path, &args.id)
        .ok_or_else(|| format!("Wallpaper '{}' not found in {}", args.id, workshop_path.display()))?;

    out::title(&format!("Wallpaper: {}", args.id));
    println!();

    // 基本信息
    out::box_start(&wallpaper.wallpaper_id);
    out::box_line("Title", wallpaper.title.as_deref().unwrap_or("(untitled)"));
    out::box_line("Type", wallpaper.wallpaper_type.as_deref().unwrap_or("unknown"));
    out::box_line("Path", &wallpaper.folder_path.display().to_string());
    out::box_line("PKG", &out::pkg_badge(wallpaper.has_pkg, Some(wallpaper.pkg_files.len())));

    if !wallpaper.pkg_files.is_empty() {
        let pkg_names: Vec<String> = wallpaper
            .pkg_files
            .iter()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        out::box_line("PKG Files", &pkg_names.join(", "));
    }

    if let Some(ref preview) = wallpaper.preview_path {
        out::box_line("Preview", &preview.display().to_string());
    }
    out::box_end();

    // 详细模式：显示每个 PKG 文件内容
    if args.verbose && !wallpaper.pkg_files.is_empty() {
        println!();
        out::subtitle("PKG Contents");

        for pkg_path in &wallpaper.pkg_files {
            let filename = pkg_path.file_name().unwrap_or_default().to_string_lossy();

            match pkg::preview_pkg(pkg_path) {
                Ok(info) => {
                    println!();
                    out::info(&format!(
                        "{} — v{} | {} files | {} TEX",
                        filename, info.version, info.file_count, info.tex_count
                    ));

                    out::table_header(&[("Name", 30), ("Size", 12), ("Type", 8)]);
                    for file in &info.files {
                        let type_str = if file.is_tex { "TEX" } else { "-" };
                        out::table_row(&[
                            (&file.name, 30),
                            (&out::format_size(file.size as u64), 12),
                            (type_str, 8),
                        ]);
                    }
                }
                Err(e) => {
                    out::error(&format!("Failed to read {}: {}", filename, e));
                }
            }
        }
    }

    // 检查处理状态
    let state = lianpkg::api::native::context::load_state_or_default(&ctx.state_path);
    if let Some(entry) = state.processed.get(&args.id) {
        println!();
        out::subtitle("Processing Status");
        out::stat("Type", format!("{:?}", entry.process_type));
        out::stat("Processed At", &entry.processed_at);
        if let Some(ref output) = entry.output_path {
            out::stat("Output", output);
        }
    }

    println!();
    Ok(())
}
