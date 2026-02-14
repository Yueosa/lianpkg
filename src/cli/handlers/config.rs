//! Config 模式处理器

use super::super::args::{ConfigArgs, ConfigCommand};
use super::super::output as out;
use lianpkg::api::native::context;
use lianpkg::core::cfg;
use std::path::PathBuf;

/// 执行 config 命令
pub fn run(args: &ConfigArgs, config_path: Option<PathBuf>) -> Result<(), String> {
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

    match &args.command {
        None | Some(ConfigCommand::Show) => show_config(&ctx),
        Some(ConfigCommand::Path) => show_path(&ctx),
        Some(ConfigCommand::Get { key }) => get_config(&ctx, key),
        Some(ConfigCommand::Set { key, value }) => set_config(&ctx, key, value),
        Some(ConfigCommand::Reset { yes }) => reset_config(&ctx, *yes),
        Some(ConfigCommand::Edit) => edit_config(&ctx),
    }
}

/// 显示完整配置
fn show_config(ctx: &context::AppContext) -> Result<(), String> {
    out::title("Configuration");

    // 读取原始 TOML 内容
    let raw_content = cfg::read_config_toml(cfg::ReadConfigInput {
        path: ctx.config_path.clone(),
    })
    .map_err(|e| e.to_string())?;

    out::path_info("Config File", &ctx.config_path);
    println!();

    // 显示解析后的配置
    let config = &ctx.config;
    out::subtitle("Wallpaper");
    out::stat("Workshop Path", config.workshop_path.display());
    out::stat("Raw Output Path", config.raw_output_path.display());
    out::stat("Enable Raw Output", config.enable_raw_output);

    println!();
    out::subtitle("Unpack");
    out::stat("Unpacked Output Path", config.unpacked_output_path.display());
    out::stat("Clean Unpacked", config.clean_unpacked);

    println!();
    out::subtitle("TEX");
    out::stat(
        "Converted Output Path",
        config
            .converted_output_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(auto)".to_string()),
    );

    println!();
    out::subtitle("Pipeline");
    out::stat("Incremental", config.pipeline.incremental);
    out::stat("Auto Unpack PKG", config.pipeline.auto_unpack_pkg);
    out::stat("Auto Convert TEX", config.pipeline.auto_convert_tex);

    println!();
    out::subtitle("Raw TOML");
    println!("{}", raw_content.content);

    Ok(())
}

/// 显示配置文件路径
fn show_path(ctx: &context::AppContext) -> Result<(), String> {
    println!("{}", ctx.config_path.display());
    Ok(())
}

/// 获取指定配置项
fn get_config(ctx: &context::AppContext, key: &str) -> Result<(), String> {
    let config = &ctx.config;

    let value = match key {
        "wallpaper.workshop_path" => config.workshop_path.display().to_string(),
        "wallpaper.raw_output_path" => config.raw_output_path.display().to_string(),
        "wallpaper.enable_raw_output" => config.enable_raw_output.to_string(),
        "unpack.unpacked_output_path" => config.unpacked_output_path.display().to_string(),
        "unpack.clean_unpacked" => config.clean_unpacked.to_string(),
        "tex.converted_output_path" => config
            .converted_output_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(not set)".to_string()),
        "pipeline.incremental" => config.pipeline.incremental.to_string(),
        "pipeline.auto_unpack_pkg" => config.pipeline.auto_unpack_pkg.to_string(),
        "pipeline.auto_convert_tex" => config.pipeline.auto_convert_tex.to_string(),
        _ => return Err(format!("Unknown config key: {}", key)),
    };

    println!("{}", value);
    Ok(())
}

/// 设置配置项
fn set_config(ctx: &context::AppContext, key: &str, value: &str) -> Result<(), String> {
    cfg::update_config_toml(cfg::UpdateConfigInput {
        path: ctx.config_path.clone(),
        key: key.to_string(),
        value: value.to_string(),
    })
    .map_err(|e| e.to_string())?;

    out::success(&format!("Set {} = {}", key, value));
    Ok(())
}

/// 重置配置
fn reset_config(ctx: &context::AppContext, yes: bool) -> Result<(), String> {
    if !yes {
        out::warning("This will reset your configuration to defaults.");
        if !out::confirm("Continue?") {
            out::info("Cancelled.");
            return Ok(());
        }
    }

    cfg::delete_config_toml(cfg::DeleteConfigInput {
        path: ctx.config_path.clone(),
    })
    .map_err(|e| e.to_string())?;

    // 重新创建默认配置
    cfg::create_config_toml(cfg::CreateConfigInput {
        path: ctx.config_path.clone(),
        content: None,
    })
    .map_err(|e| e.to_string())?;

    out::success("Configuration reset to defaults.");
    Ok(())
}

/// 用编辑器打开配置文件
fn edit_config(ctx: &context::AppContext) -> Result<(), String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "notepad".to_string()
        } else {
            "vi".to_string()
        }
    });

    out::info(&format!(
        "Opening {} with {}...",
        ctx.config_path.display(),
        editor
    ));

    std::process::Command::new(&editor)
        .arg(&ctx.config_path)
        .status()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor, e))?;

    Ok(())
}
