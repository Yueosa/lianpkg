//! Config 模式处理器

use std::path::PathBuf;
use std::process::Command;
use super::super::args::{ConfigArgs, ConfigCommand};
use super::super::output as out;
use lianpkg::api::native;
use lianpkg::core::{cfg, path};

/// 执行 config 命令
pub fn run(args: &ConfigArgs, config_path: Option<PathBuf>) -> Result<(), String> {
    // 确定配置目录
    let config_dir = config_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf());

    let init_result = native::init_config(native::InitConfigInput {
        config_dir,
        use_exe_dir: config_path.is_none(),
    });

    match &args.command {
        Some(ConfigCommand::Show) => show_config(&init_result.config_path),
        Some(ConfigCommand::Path) => show_path(&init_result.config_path, &init_result.state_path),
        Some(ConfigCommand::Get { key }) => get_config(&init_result.config_path, key),
        Some(ConfigCommand::Set { key, value }) => set_config(&init_result.config_path, key, value),
        Some(ConfigCommand::Reset { yes }) => reset_config(&init_result.config_path, *yes),
        Some(ConfigCommand::Edit) => edit_config(&init_result.config_path),
        None => show_config(&init_result.config_path),
    }
}

/// 显示完整配置
fn show_config(config_path: &PathBuf) -> Result<(), String> {
    out::title("Configuration");
    out::path_info("Config File", config_path);
    println!();

    // 读取并显示配置
    let read_result = cfg::read_config_toml(cfg::ReadConfigInput {
        path: config_path.clone(),
    });

    if !read_result.success {
        return Err("Failed to read config".to_string());
    }

    let content = read_result.content.unwrap_or_default();
    
    // 解析并格式化显示
    let load_result = native::load_config(native::LoadConfigInput {
        config_path: config_path.clone(),
    });

    if let Some(config) = load_result.config {
        out::subtitle("[wallpaper]");
        out::stat("workshop_path", config.workshop_path.display());
        out::stat("raw_output_path", config.raw_output_path.display());
        out::stat("enable_raw_output", config.enable_raw_output);
        out::stat("pkg_temp_path", config.pkg_temp_path.display());

        out::subtitle("[unpack]");
        out::stat("unpacked_output_path", config.unpacked_output_path.display());
        out::stat("clean_pkg_temp", config.clean_pkg_temp);
        out::stat("clean_unpacked", config.clean_unpacked);

        out::subtitle("[tex]");
        out::stat("converted_output_path", 
            config.converted_output_path
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(auto)".to_string())
        );

        out::subtitle("[pipeline]");
        out::stat("incremental", config.pipeline.incremental);
        out::stat("auto_unpack_pkg", config.pipeline.auto_unpack_pkg);
        out::stat("auto_convert_tex", config.pipeline.auto_convert_tex);
    } else {
        // 直接显示原始内容
        println!("{}", content);
    }

    Ok(())
}

/// 显示配置文件路径
fn show_path(config_path: &PathBuf, state_path: &PathBuf) -> Result<(), String> {
    out::title("Configuration Paths");
    out::path_info("Config File", config_path);
    out::path_info("State File", state_path);
    out::path_info("Config Directory", config_path.parent().unwrap_or(config_path));
    
    // 默认路径
    out::subtitle("Default Paths");
    out::stat("Default Config Dir", path::default_config_dir().display());
    out::stat("Default Workshop", path::default_workshop_path());
    out::stat("Default Raw Output", path::default_raw_output_path());
    out::stat("Default PKG Temp", path::default_pkg_temp_path());
    out::stat("Default Unpacked", path::default_unpacked_output_path());

    Ok(())
}

/// 获取指定配置项
fn get_config(config_path: &PathBuf, key: &str) -> Result<(), String> {
    let read_result = cfg::read_config_toml(cfg::ReadConfigInput {
        path: config_path.clone(),
    });

    if !read_result.success {
        return Err("Failed to read config".to_string());
    }

    let content = read_result.content.unwrap_or_default();
    
    // 解析 TOML
    let doc: toml::Table = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // 按点分隔的键查找
    let parts: Vec<&str> = key.split('.').collect();
    let value = find_value(&doc, &parts);

    match value {
        Some(v) => {
            println!("{}", format_toml_value(&v));
            Ok(())
        }
        None => Err(format!("Key '{}' not found", key)),
    }
}

/// 设置配置项
fn set_config(config_path: &PathBuf, key: &str, value: &str) -> Result<(), String> {
    let result = cfg::update_config_toml(cfg::UpdateConfigInput {
        path: config_path.clone(),
        key: key.to_string(),
        value: value.to_string(),
    });

    if result.success {
        out::success(&format!("Set {} = {}", key, value));
        Ok(())
    } else {
        Err("Failed to update config".to_string())
    }
}

/// 重置配置
fn reset_config(config_path: &PathBuf, yes: bool) -> Result<(), String> {
    if !yes {
        out::warning("This will reset config.toml to default values");
        if !out::confirm("Are you sure?") {
            return Err("Operation cancelled".to_string());
        }
    }

    // 删除现有配置
    let _ = cfg::delete_config_toml(cfg::DeleteConfigInput {
        path: config_path.clone(),
    });

    // 重新创建
    let result = cfg::create_config_toml(cfg::CreateConfigInput {
        path: config_path.clone(),
        content: None,
    });

    if result.created {
        out::success("Config reset to defaults");
        Ok(())
    } else {
        Err("Failed to reset config".to_string())
    }
}

/// 用编辑器打开配置
fn edit_config(config_path: &PathBuf) -> Result<(), String> {
    // 获取编辑器
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            #[cfg(windows)]
            { "notepad".to_string() }
            #[cfg(not(windows))]
            { "vi".to_string() }
        });

    out::info(&format!("Opening config with {}", editor));

    let status = Command::new(&editor)
        .arg(config_path)
        .status()
        .map_err(|e| format!("Failed to open editor: {}", e))?;

    if status.success() {
        out::success("Config edited");
        Ok(())
    } else {
        Err("Editor exited with error".to_string())
    }
}

/// 在 TOML 表中查找值
fn find_value<'a>(table: &'a toml::Table, parts: &[&str]) -> Option<&'a toml::Value> {
    if parts.is_empty() {
        return None;
    }

    let mut current: &toml::Value = table.get(parts[0])?;
    
    for part in &parts[1..] {
        match current {
            toml::Value::Table(t) => {
                current = t.get(*part)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

/// 格式化 TOML 值为字符串
fn format_toml_value(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_toml_value).collect();
            format!("[{}]", items.join(", "))
        }
        toml::Value::Table(t) => {
            let items: Vec<String> = t.iter()
                .map(|(k, v)| format!("{} = {}", k, format_toml_value(v)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
        toml::Value::Datetime(dt) => dt.to_string(),
    }
}
