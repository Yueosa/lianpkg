//! TEX 模式处理器

use super::super::args::TexArgs;
use super::super::output as out;
use lianpkg::api::native::{context, tex};
use lianpkg::core::path;
use std::fs;
use std::path::PathBuf;

/// 执行 tex 命令
pub fn run(args: &TexArgs, config_path: Option<PathBuf>) -> Result<(), String> {
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

    // 确定路径
    let input_path = args
        .path
        .clone()
        .unwrap_or_else(|| ctx.config.unpacked_output_path.clone());

    let output_path = args.output.clone().or_else(|| ctx.config.converted_output_path.clone());

    if !input_path.exists() {
        return Err(format!(
            "Input path does not exist: {}",
            input_path.display()
        ));
    }

    // 预览模式
    if args.preview {
        return run_preview(&input_path, args.verbose);
    }

    // 执行转换
    out::title("TEX Convert");
    out::path_info("Input", &input_path);
    if let Some(ref out_path) = output_path {
        out::path_info("Output", out_path);
    }
    println!();

    if input_path.is_file()
        && input_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase() == "tex")
            .unwrap_or(false)
    {
        // 单文件转换
        let out_path = output_path.unwrap_or_else(|| {
            let mut p = input_path.clone();
            p.set_extension("png");
            p
        });

        let _ = path::ensure_dir_compat(out_path.parent().unwrap_or(&out_path));

        out::debug_api_enter(
            "tex",
            "convert_single",
            &format!("input={}", input_path.display()),
        );
        let result = tex::convert_single(&input_path, &out_path)
            .map_err(|e| e.to_string())?;

        if !result.success {
            out::debug_api_error(result.error.as_deref().unwrap_or("Unknown error"));
            return Err(result.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        out::debug_api_return(&format!(
            "format={}, output={}",
            result.format.as_deref().unwrap_or("?"),
            result.output_path.display()
        ));

        out::subtitle("Results");
        if let Some(ref info) = result.tex_info {
            out::stat("Format", &info.format);
            out::stat("Size", format!("{}x{}", info.width, info.height));
            out::stat("Output", result.format.as_deref().unwrap_or("?"));
        }
        println!();
        out::success("TEX conversion completed!");
    } else {
        // 目录批量转换
        let _ = output_path
            .as_ref()
            .map(|p| path::ensure_dir_compat(p));

        out::debug_api_enter(
            "tex",
            "convert_all",
            &format!("input={}", input_path.display()),
        );
        let result = tex::convert_all(&input_path, output_path.as_deref())
            .map_err(|e| e.to_string())?;

        out::debug_api_return(&format!(
            "processed={}, success={}, failed={}",
            result.stats.tex_processed, result.stats.tex_success, result.stats.tex_failed
        ));

        out::subtitle("Results");
        out::stat("TEX Processed", result.stats.tex_processed);
        out::stat("TEX Success", result.stats.tex_success);
        out::stat("TEX Failed", result.stats.tex_failed);
        out::stat("TEX Skipped", result.stats.tex_skipped);
        out::stat("Images", result.stats.image_count);
        out::stat("Videos", result.stats.video_count);
        println!();

        if result.stats.tex_failed > 0 {
            out::warning(&format!(
                "{} TEX files failed to convert",
                result.stats.tex_failed
            ));
        }
        out::success("TEX conversion completed!");
    }

    Ok(())
}

/// 预览模式
fn run_preview(input_path: &PathBuf, verbose: bool) -> Result<(), String> {
    out::title("TEX Preview");
    out::path_info("Input", input_path);
    println!();

    if input_path.is_file() {
        preview_single_tex(input_path, verbose)?;
    } else {
        preview_directory(input_path, verbose)?;
    }

    Ok(())
}

/// 预览单个 TEX 文件
fn preview_single_tex(tex_path: &std::path::Path, verbose: bool) -> Result<(), String> {
    let info = tex::preview_tex(tex_path).map_err(|e| e.to_string())?;

    if verbose {
        out::box_start(&tex_path.file_name().unwrap_or_default().to_string_lossy());
        out::box_line("Version", &info.version);
        out::box_line("Format", &info.format);
        out::box_line("Size", &format!("{}x{}", info.width, info.height));
        out::box_line("Images", &info.image_count.to_string());
        out::box_line("Mipmaps", &info.mipmap_count.to_string());
        out::box_line("Compressed", if info.is_compressed { "Yes" } else { "No" });
        out::box_line("Video", if info.is_video { "Yes" } else { "No" });
        out::box_line("Data Size", &out::format_size(info.data_size as u64));
        out::box_line("Recommended", &info.recommended_output);
        out::box_end();
    } else {
        out::info(&format!(
            "{} | {}x{} | {} | {}",
            info.format,
            info.width,
            info.height,
            if info.is_video { "video" } else { "image" },
            out::format_size(info.data_size as u64)
        ));
    }

    println!();
    Ok(())
}

/// 预览目录中的所有 TEX
fn preview_directory(dir_path: &PathBuf, verbose: bool) -> Result<(), String> {
    let tex_files = find_tex_files_recursive(dir_path)?;

    if tex_files.is_empty() {
        out::warning("No TEX files found in directory");
        return Ok(());
    }

    out::info(&format!("Found {} TEX files", tex_files.len()));
    println!();

    if verbose {
        for tex_path in &tex_files {
            if let Err(e) = preview_single_tex(tex_path, true) {
                out::error(&format!(
                    "Failed to preview {}: {}",
                    tex_path.display(),
                    e
                ));
            }
        }
    } else {
        out::table_header(&[
            ("File", 30),
            ("Format", 12),
            ("Size", 12),
            ("Type", 8),
        ]);

        for tex_path in &tex_files {
            match tex::preview_tex(tex_path) {
                Ok(info) => {
                    let filename = tex_path.file_name().unwrap_or_default().to_string_lossy();
                    let size_str = format!("{}x{}", info.width, info.height);
                    let type_str = if info.is_video { "video" } else { "image" };
                    out::table_row(&[
                        (&filename, 30),
                        (&info.format, 12),
                        (&size_str, 12),
                        (type_str, 8),
                    ]);
                }
                Err(e) => {
                    let filename = tex_path.file_name().unwrap_or_default().to_string_lossy();
                    out::table_row(&[
                        (&filename, 30),
                        ("error", 12),
                        (&e.to_string(), 12),
                        ("-", 8),
                    ]);
                }
            }
        }
    }

    println!();
    Ok(())
}

/// 递归查找目录中的 TEX 文件
fn find_tex_files_recursive(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut tex_files = Vec::new();
    collect_tex_files(dir, &mut tex_files)
        .map_err(|e| format!("Failed to scan directory: {}", e))?;
    Ok(tex_files)
}

fn collect_tex_files(dir: &std::path::Path, result: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_tex_files(&path, result)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "tex" {
                    result.push(path);
                }
            }
        }
    }
    Ok(())
}
