//! config.toml 文件操作接口

use std::fs;
use toml::Value;

use crate::core::cfg::structs::{
    CreateConfigInput, CreateConfigOutput,
    ReadConfigInput, ReadConfigOutput,
    UpdateConfigInput, UpdateConfigOutput,
    DeleteConfigInput, DeleteConfigOutput,
};
use crate::core::cfg::utl::{default_config_template, ensure_dir};

/// 创建配置文件
/// 如果文件已存在则不创建，返回 created = false
/// 如果不提供内容则使用默认模板
pub fn create_config_toml(input: CreateConfigInput) -> CreateConfigOutput {
    let path = input.path;
    
    // 文件已存在，不触发创建
    if path.exists() {
        return CreateConfigOutput {
            created: false,
            path,
        };
    }
    
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        if let Err(_) = ensure_dir(parent) {
            return CreateConfigOutput {
                created: false,
                path,
            };
        }
    }
    
    // 获取内容：优先使用提供的内容，否则使用默认模板
    let content = input.content.unwrap_or_else(|| default_config_template());
    
    // 写入文件
    match fs::write(&path, content) {
        Ok(_) => CreateConfigOutput {
            created: true,
            path,
        },
        Err(_) => CreateConfigOutput {
            created: false,
            path,
        },
    }
}

/// 读取配置文件
/// 返回文件内容，文件不存在或读取失败时 success = false
pub fn read_config_toml(input: ReadConfigInput) -> ReadConfigOutput {
    let path = input.path;
    
    if !path.exists() {
        return ReadConfigOutput {
            success: false,
            content: None,
        };
    }
    
    match fs::read_to_string(&path) {
        Ok(content) => ReadConfigOutput {
            success: true,
            content: Some(content),
        },
        Err(_) => ReadConfigOutput {
            success: false,
            content: None,
        },
    }
}

/// 更新配置文件
/// 支持点号分隔的嵌套键，如 "wallpaper.workshop_path"
/// 键存在则更新，不存在则新建
pub fn update_config_toml(input: UpdateConfigInput) -> UpdateConfigOutput {
    let path = input.path.clone();
    
    // 读取现有内容
    let read_result = read_config_toml(ReadConfigInput { path: path.clone() });
    
    let content = match read_result.content {
        Some(c) => c,
        None => {
            return UpdateConfigOutput {
                success: false,
                content: None,
            };
        }
    };
    
    // 解析 TOML
    let mut value: Value = match content.parse() {
        Ok(v) => v,
        Err(_) => {
            return UpdateConfigOutput {
                success: false,
                content: None,
            };
        }
    };
    
    // 解析键路径并更新值
    let keys: Vec<&str> = input.key.split('.').collect();
    if !set_nested_value(&mut value, &keys, &input.value) {
        return UpdateConfigOutput {
            success: false,
            content: None,
        };
    }
    
    // 序列化并写回
    let new_content = match toml::to_string_pretty(&value) {
        Ok(s) => s,
        Err(_) => {
            return UpdateConfigOutput {
                success: false,
                content: None,
            };
        }
    };
    
    if fs::write(&path, &new_content).is_err() {
        return UpdateConfigOutput {
            success: false,
            content: None,
        };
    }
    
    // 复用 read_config_toml 返回最新内容
    let final_read = read_config_toml(ReadConfigInput { path });
    UpdateConfigOutput {
        success: true,
        content: final_read.content,
    }
}

/// 删除配置文件
/// 文件不存在视为成功，但 deleted = false
pub fn delete_config_toml(input: DeleteConfigInput) -> DeleteConfigOutput {
    let path = input.path;
    
    // 文件不存在，不触发删除
    if !path.exists() {
        return DeleteConfigOutput {
            deleted: false,
            path,
        };
    }
    
    // 删除文件
    match fs::remove_file(&path) {
        Ok(_) => DeleteConfigOutput {
            deleted: true,
            path,
        },
        Err(_) => DeleteConfigOutput {
            deleted: false,
            path,
        },
    }
}

/// 设置嵌套键的值
/// 支持点号分隔的键路径
fn set_nested_value(root: &mut Value, keys: &[&str], new_value: &str) -> bool {
    if keys.is_empty() {
        return false;
    }
    
    // 尝试解析新值为合适的 TOML 类型
    let parsed_value = parse_value_string(new_value);
    
    if keys.len() == 1 {
        // 最后一层，直接设置值
        if let Value::Table(table) = root {
            table.insert(keys[0].to_string(), parsed_value);
            return true;
        }
        return false;
    }
    
    // 递归处理中间层
    if let Value::Table(table) = root {
        let key = keys[0];
        let entry = table.entry(key.to_string())
            .or_insert_with(|| Value::Table(toml::map::Map::new()));
        return set_nested_value(entry, &keys[1..], new_value);
    }
    
    false
}

/// 将字符串值解析为合适的 TOML Value 类型
fn parse_value_string(s: &str) -> Value {
    // 尝试解析为布尔值
    if s == "true" {
        return Value::Boolean(true);
    }
    if s == "false" {
        return Value::Boolean(false);
    }
    
    // 尝试解析为整数
    if let Ok(i) = s.parse::<i64>() {
        return Value::Integer(i);
    }
    
    // 尝试解析为浮点数
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }
    
    // 默认作为字符串
    Value::String(s.to_string())
}
