//! PKG 模式处理器

use super::super::args::PkgArgs;
use super::super::output as out;
use lianpkg::api::native::{self, pkg};
use lianpkg::core::path;
use std::fs;
use std::path::PathBuf;

/// 执行 pkg 命令
pub fn run(args: &PkgArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    // 加载配置
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
    let input_path = args
        .path
        .clone()
        .unwrap_or_else(|| config.pkg_temp_path.clone());

    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| config.unpacked_output_path.clone());

    // 判断输入类型
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

    // 执行解包
    out::title("PKG Unpack");
    out::path_info("Input", &input_path);
    out::path_info("Output", &output_path);
    println!();

    // 确保输出目录存在
    let _ = path::ensure_dir_compat(&output_path);

    // 判断是单文件还是目录
    if input_path.is_file() && input_path.extension().map(|e| e == "pkg").unwrap_or(false) {
        // 单文件解包
        out::debug_api_enter(
            "pkg",
            "unpack_single",
            &format!("input={}", input_path.display()),
        );
        let result = pkg::unpack_single(input_path.clone(), output_path);

        if !result.success {
            out::debug_api_error(result.error.as_deref().unwrap_or("Unknown error"));
            return Err(result.error.unwrap_or_else(|| "Unknown error".to_string()));
        }
        out::debug_api_return(&format!(
            "scene={}, files={}",
            result.scene_name,
            result.files.len()
        ));

        out::subtitle("Results");
        out::stat("Scene", &result.scene_name);
        out::stat("Files Extracted", result.files.len());

        let tex_count = result.files.iter().filter(|f| f.is_tex).count();
        out::stat("TEX Files", tex_count);
        println!();
        out::success("PKG unpack completed!");
    } else {
        // 目录批量解包
        out::debug_api_enter(
            "pkg",
            "unpack_all",
            &format!("input={}", input_path.display()),
        );
        let result = pkg::unpack_all(pkg::UnpackAllInput {
            pkg_temp_path: input_path,
            unpacked_output_path: output_path,
        });

        if !result.success && result.stats.pkg_success == 0 {
            out::debug_api_error(result.error.as_deref().unwrap_or("Unknown error"));
            return Err(result.error.unwrap_or_else(|| "Unknown error".to_string()));
        }
        out::debug_api_return(&format!(
            "processed={}, success={}, failed={}",
            result.stats.pkg_processed, result.stats.pkg_success, result.stats.pkg_failed
        ));

        out::subtitle("Results");
        out::stat("PKGs Processed", result.stats.pkg_processed);
        out::stat("PKGs Success", result.stats.pkg_success);
        out::stat("PKGs Failed", result.stats.pkg_failed);
        out::stat("Total Files", result.stats.total_files);
        out::stat("TEX Files", result.stats.tex_files);
        println!();

        if result.stats.pkg_failed > 0 {
            out::warning(&format!(
                "{} PKG files failed to unpack",
                result.stats.pkg_failed
            ));
        }
        out::success("PKG unpack completed!");
    }

    Ok(())
}

/// 预览模式
fn run_preview(input_path: &PathBuf, verbose: bool) -> Result<(), String> {
    out::title("PKG Preview");
    out::path_info("Input", input_path);
    println!();

    if input_path.is_file() {
        // 单文件预览
        preview_single_pkg(input_path, verbose)?;
    } else {
        // 目录预览
        preview_directory(input_path, verbose)?;
    }

    Ok(())
}

/// 预览单个 PKG 文件
fn preview_single_pkg(pkg_path: &std::path::Path, verbose: bool) -> Result<(), String> {
    let result = pkg::preview_pkg(pkg::PreviewPkgInput {
        pkg_path: pkg_path.to_path_buf(),
    });

    if !result.success {
        return Err(result
            .error
            .unwrap_or_else(|| "Failed to parse PKG".to_string()));
    }

    let info = result.pkg_info.ok_or("PKG info is empty")?;

    out::info(&format!(
        "Version: {} | Files: {} | TEX: {}",
        info.version, info.file_count, info.tex_count
    ));
    println!();

    if verbose {
        out::subtitle("Files");
        for file in &info.files {
            let tex_mark = if file.is_tex {
                out::tex_badge(true)
            } else {
                String::new()
            };
            println!(
                "    {:30} {:>10}  {}",
                file.name,
                out::format_size(file.size as u64),
                tex_mark
            );
        }
    } else {
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

    println!();
    Ok(())
}

/// 预览目录中的所有 PKG
fn preview_directory(dir_path: &PathBuf, verbose: bool) -> Result<(), String> {
    let pkg_files = find_pkg_files(dir_path)?;

    if pkg_files.is_empty() {
        out::warning("No PKG files found in directory");
        return Ok(());
    }

    out::info(&format!("Found {} PKG files", pkg_files.len()));
    println!();

    if verbose {
        // 详细模式：每个 PKG 单独显示
        for pkg_path in &pkg_files {
            out::subtitle(&pkg_path.file_name().unwrap_or_default().to_string_lossy());
            if let Err(e) = preview_single_pkg(pkg_path, false) {
                out::error(&format!("Failed to preview: {}", e));
            }
        }
    } else {
        // 简洁模式：表格汇总
        out::table_header(&[("File", 35), ("Version", 10), ("Files", 8), ("TEX", 6)]);

        for pkg_path in &pkg_files {
            let result = pkg::preview_pkg(pkg::PreviewPkgInput {
                pkg_path: pkg_path.clone(),
            });

            if result.success {
                if let Some(info) = result.pkg_info {
                    let filename = pkg_path.file_name().unwrap_or_default().to_string_lossy();

                    out::table_row(&[
                        (&filename, 35),
                        (&info.version, 10),
                        (&info.file_count.to_string(), 8),
                        (&info.tex_count.to_string(), 6),
                    ]);
                }
            } else {
                let filename = pkg_path.file_name().unwrap_or_default().to_string_lossy();
                out::table_row(&[(&filename, 35), ("ERROR", 10), ("-", 8), ("-", 6)]);
            }
        }
    }

    println!();
    Ok(())
}

/// 递归查找目录中的 PKG 文件
fn find_pkg_files(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut pkg_files = Vec::new();

    let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "pkg" {
                    pkg_files.push(path);
                }
            }
        } else if path.is_dir() {
            // 递归搜索
            if let Ok(sub_files) = find_pkg_files(&path) {
                pkg_files.extend(sub_files);
            }
        }
    }

    Ok(pkg_files)
}
