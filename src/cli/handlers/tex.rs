//! TEX 模式处理器

use std::path::PathBuf;
use std::fs;
use super::super::args::TexArgs;
use super::super::output as out;
use lianpkg::api::native::{self, tex};
use lianpkg::core::path;

/// 执行 tex 命令
pub fn run(args: &TexArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    // 加载配置
    let init_result = native::init_config(native::InitConfigInput {
        config_dir: config_path.map(|p| p.parent().unwrap_or(&p).to_path_buf()),
    });

    let config_result = native::load_config(native::LoadConfigInput {
        config_path: init_result.config_path.clone(),
    });

    let config = config_result.config
        .ok_or("Failed to load config")?;

    // 确定路径
    let input_path = args.path.clone()
        .unwrap_or_else(|| config.unpacked_output_path.clone());

    let output_path = args.output.clone()
        .or(config.converted_output_path.clone());

    // 判断输入类型
    if !input_path.exists() {
        return Err(format!("Input path does not exist: {}", input_path.display()));
    }

    // 预览模式
    if args.preview {
        return run_preview(&input_path, args.verbose);
    }

    // 执行转换
    out::title("TEX Conversion");
    out::path_info("Input", &input_path);
    if let Some(ref out_path) = output_path {
        out::path_info("Output", out_path);
    } else {
        out::info("Output: (auto - tex_converted subdirectory)");
    }
    println!();

    // 确保输出目录存在
    if let Some(ref out_path) = output_path {
        let _ = path::ensure_dir(out_path);
    }

    // 判断是单文件还是目录
    if input_path.is_file() && input_path.extension().map(|e| e == "tex").unwrap_or(false) {
        // 单文件转换
        let out_path = output_path.unwrap_or_else(|| {
            input_path.parent().unwrap_or(&input_path).join("tex_converted")
        });
        
        let result = tex::convert_single(input_path.clone(), out_path);
        
        if !result.success {
            return Err(result.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        out::subtitle("Results");
        out::stat("Output", result.output_path.display());
        out::stat("Format", result.format.as_deref().unwrap_or("unknown"));
        if let Some(info) = result.tex_info {
            out::stat("Resolution", format!("{}×{}", info.width, info.height));
        }
        println!();
        out::success("TEX conversion completed!");
    } else {
        // 目录批量转换
        let result = tex::convert_all(tex::ConvertAllInput {
            unpacked_path: input_path,
            output_path,
        });

        if !result.success && result.stats.tex_success == 0 {
            return Err(result.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        out::subtitle("Results");
        out::stat("TEX Processed", result.stats.tex_processed);
        out::stat("TEX Success", result.stats.tex_success);
        out::stat("TEX Failed", result.stats.tex_failed);
        out::stat("Images", result.stats.image_count);
        out::stat("Videos", result.stats.video_count);
        println!();

        if result.stats.tex_failed > 0 {
            out::warning(&format!("{} TEX files failed to convert", result.stats.tex_failed));
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
        // 单文件预览
        preview_single_tex(input_path, verbose)?;
    } else {
        // 目录预览
        preview_directory(input_path, verbose)?;
    }

    Ok(())
}

/// 预览单个 TEX 文件
fn preview_single_tex(tex_path: &PathBuf, verbose: bool) -> Result<(), String> {
    let result = tex::preview_tex(tex::PreviewTexInput {
        tex_path: tex_path.clone(),
    });

    if !result.success {
        return Err(result.error.unwrap_or_else(|| "Failed to parse TEX".to_string()));
    }

    let info = result.tex_info.ok_or("TEX info is empty")?;

    if verbose {
        out::box_start(&tex_path.file_name().unwrap_or_default().to_string_lossy());
        out::box_line("Path", &tex_path.display().to_string());
        out::box_line("Version", &info.version);
        out::box_line("Format", &info.format);
        out::box_line("Size", &format!("{} × {}", info.width, info.height));
        out::box_line("Images", &info.image_count.to_string());
        out::box_line("Mipmaps", &info.mipmap_count.to_string());
        out::box_line("Compressed", if info.is_compressed { "Yes (LZ4)" } else { "No" });
        out::box_line("Video", if info.is_video { "Yes" } else { "No" });
        out::box_line("Data Size", &out::format_size(info.data_size as u64));
        out::box_line("Output", &format!("→ {}", info.recommended_output.to_uppercase()));
        out::box_end();
    } else {
        let filename = tex_path.file_name().unwrap_or_default().to_string_lossy();
        out::info(&format!(
            "{}: {} | {}×{} | {} | → {}",
            filename,
            info.format,
            info.width,
            info.height,
            if info.is_compressed { "LZ4" } else { "Raw" },
            info.recommended_output.to_uppercase()
        ));
    }

    Ok(())
}

/// 预览目录中的所有 TEX
fn preview_directory(dir_path: &PathBuf, verbose: bool) -> Result<(), String> {
    let tex_files = find_tex_files(dir_path)?;
    
    if tex_files.is_empty() {
        out::warning("No TEX files found in directory");
        return Ok(());
    }

    out::info(&format!("Found {} TEX files", tex_files.len()));
    println!();

    if verbose {
        // 详细模式：每个 TEX 单独显示
        for tex_path in &tex_files {
            if let Err(e) = preview_single_tex(tex_path, true) {
                out::error(&format!("Failed to preview {}: {}", 
                    tex_path.file_name().unwrap_or_default().to_string_lossy(),
                    e
                ));
            }
            println!();
        }
    } else {
        // 简洁模式：表格汇总
        out::table_header(&[
            ("File", 25),
            ("Format", 8),
            ("Size", 12),
            ("LZ4", 5),
            ("Output", 6),
        ]);

        for tex_path in &tex_files {
            let result = tex::preview_tex(tex::PreviewTexInput {
                tex_path: tex_path.clone(),
            });

            if result.success {
                if let Some(info) = result.tex_info {
                    let filename = tex_path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    let size = format!("{}×{}", info.width, info.height);
                    let compressed = if info.is_compressed { "✓" } else { "✗" };
                    
                    out::table_row(&[
                        (&filename, 25),
                        (&info.format, 8),
                        (&size, 12),
                        (compressed, 5),
                        (&info.recommended_output.to_uppercase(), 6),
                    ]);
                }
            } else {
                let filename = tex_path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                out::table_row(&[
                    (&filename, 25),
                    ("ERROR", 8),
                    ("-", 12),
                    ("-", 5),
                    ("-", 6),
                ]);
            }
        }
    }

    println!();
    Ok(())
}

/// 递归查找目录中的 TEX 文件
fn find_tex_files(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut tex_files = Vec::new();

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "tex" {
                    tex_files.push(path);
                }
            }
        } else if path.is_dir() {
            // 递归搜索
            if let Ok(sub_files) = find_tex_files(&path) {
                tex_files.extend(sub_files);
            }
        }
    }

    Ok(tex_files)
}
