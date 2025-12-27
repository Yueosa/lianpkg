//! Config 路径解析

use std::path::PathBuf;

/// 获取默认配置目录
/// - Linux: ~/.config/lianpkg
/// - Windows: %APPDATA%/lianpkg
pub fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("lianpkg")
}

/// 获取默认 config.toml 路径
pub fn default_config_toml_path() -> PathBuf {
    default_config_dir().join("config.toml")
}

/// 获取默认 state.json 路径
pub fn default_state_json_path() -> PathBuf {
    default_config_dir().join("state.json")
}
