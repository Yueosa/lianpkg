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

/// 获取 exe 所在目录（仅 Windows，失败返回 None）
#[cfg(target_os = "windows")]
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// Windows: 获取 exe 同目录的配置目录
#[cfg(target_os = "windows")]
pub fn exe_config_dir() -> Option<PathBuf> {
    exe_dir().map(|p| p.join("config"))
}

/// 非 Windows 平台的空实现
#[cfg(not(target_os = "windows"))]
pub fn exe_dir() -> Option<PathBuf> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn exe_config_dir() -> Option<PathBuf> {
    None
}
