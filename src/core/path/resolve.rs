//! 统一路径解析接口
//!
//! 将多个路径生成函数合并为单一 `resolve_path` 接口

use crate::core::error::CoreResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// 路径类型枚举
// ============================================================================

/// 路径类型枚举
#[derive(Debug, Clone)]
pub enum PathType {
    /// 配置目录 (~/.config/lianpkg)
    ConfigDir,
    /// config.toml 文件路径
    ConfigToml,
    /// state.json 文件路径
    StateJson,
    /// Steam Workshop 路径
    Workshop,
    /// 原始壁纸输出路径
    RawOutput,
    /// PKG 临时路径
    PkgTemp,
    /// 解包输出路径
    UnpackedOutput,
    /// PKG 临时目标名 (dir_name + file_name)
    PkgTempDest { dir_name: String, file_name: String },
    /// 从 PKG stem 提取场景名
    SceneName { stem: String },
    /// TEX 输出目录
    TexOutput {
        tex_path: PathBuf,
        output_base: PathBuf,
    },
}

// ============================================================================
// Input/Output 结构体
// ============================================================================

/// resolve_path 接口入参
#[derive(Debug, Clone)]
pub struct ResolvePathInput {
    /// 路径类型
    pub path_type: PathType,
}

/// resolve_path 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvePathOutput {
    /// 解析后的路径（PathBuf 或 String）
    pub path: PathBuf,
    /// 路径字符串形式（用于配置文件写入）
    pub path_str: String,
}

// ============================================================================
// 路径解析实现
// ============================================================================

/// 统一路径解析入口
pub fn resolve_path(input: ResolvePathInput) -> CoreResult<ResolvePathOutput> {
    match input.path_type {
        PathType::ConfigDir => resolve_config_dir(),
        PathType::ConfigToml => resolve_config_toml(),
        PathType::StateJson => resolve_state_json(),
        PathType::Workshop => resolve_workshop(),
        PathType::RawOutput => resolve_raw_output(),
        PathType::PkgTemp => resolve_pkg_temp(),
        PathType::UnpackedOutput => resolve_unpacked_output(),
        PathType::PkgTempDest {
            dir_name,
            file_name,
        } => resolve_pkg_temp_dest(&dir_name, &file_name),
        PathType::SceneName { stem } => resolve_scene_name(&stem),
        PathType::TexOutput {
            tex_path,
            output_base,
        } => resolve_tex_output(&tex_path, &output_base),
    }
}

// ============================================================================
// 内部实现
// ============================================================================

fn resolve_config_dir() -> CoreResult<ResolvePathOutput> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("lianpkg");
    let path_str = path.display().to_string();
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_config_toml() -> CoreResult<ResolvePathOutput> {
    let config_dir = resolve_config_dir()?.path;
    let path = config_dir.join("config.toml");
    let path_str = path.display().to_string();
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_state_json() -> CoreResult<ResolvePathOutput> {
    let config_dir = resolve_config_dir()?.path;
    let path = config_dir.join("state.json");
    let path_str = path.display().to_string();
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_workshop() -> CoreResult<ResolvePathOutput> {
    let path_str = get_workshop_path_impl();
    let path = PathBuf::from(&path_str);
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_raw_output() -> CoreResult<ResolvePathOutput> {
    let path_str = {
        #[cfg(target_os = "windows")]
        {
            if let Some(exe_dir) = get_exe_dir() {
                exe_dir.join("Wallpapers_Raw").display().to_string()
            } else {
                windows_appdata_path("Wallpapers_Raw")
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            "~/.local/share/lianpkg/Wallpapers_Raw".to_string()
        }
    };
    let path = PathBuf::from(&path_str);
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_pkg_temp() -> CoreResult<ResolvePathOutput> {
    let path_str = {
        #[cfg(target_os = "windows")]
        {
            if let Some(exe_dir) = get_exe_dir() {
                exe_dir.join("Pkg_Temp").display().to_string()
            } else {
                windows_appdata_path("Pkg_Temp")
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            "~/.local/share/lianpkg/Pkg_Temp".to_string()
        }
    };
    let path = PathBuf::from(&path_str);
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_unpacked_output() -> CoreResult<ResolvePathOutput> {
    let path_str = {
        #[cfg(target_os = "windows")]
        {
            if let Some(exe_dir) = get_exe_dir() {
                exe_dir.join("Pkg_Unpacked").display().to_string()
            } else {
                windows_appdata_path("Pkg_Unpacked")
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            "~/.local/share/lianpkg/Pkg_Unpacked".to_string()
        }
    };
    let path = PathBuf::from(&path_str);
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_pkg_temp_dest(dir_name: &str, file_name: &str) -> CoreResult<ResolvePathOutput> {
    let path_str = format!("{}_{}", dir_name, file_name);
    let path = PathBuf::from(&path_str);
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_scene_name(stem: &str) -> CoreResult<ResolvePathOutput> {
    let path_str = if let Some((prefix, _)) = stem.split_once('_') {
        prefix.to_string()
    } else {
        stem.to_string()
    };
    let path = PathBuf::from(&path_str);
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_tex_output(
    tex_path: &std::path::Path,
    output_base: &std::path::Path,
) -> CoreResult<ResolvePathOutput> {
    let base_dir = output_base
        .join(tex_path.file_stem().unwrap_or_default())
        .join("tex_converted");

    // 尝试保持目录结构
    let path = if let Some(parent) = tex_path.parent() {
        if let Ok(relative) = tex_path.strip_prefix(parent) {
            if let Some(rel_parent) = relative.parent() {
                if rel_parent.components().count() > 0 {
                    base_dir.join(rel_parent)
                } else {
                    base_dir
                }
            } else {
                base_dir
            }
        } else {
            base_dir
        }
    } else {
        base_dir
    };

    let path_str = path.display().to_string();
    Ok(ResolvePathOutput { path, path_str })
}

// ============================================================================
// 平台相关辅助函数
// ============================================================================

#[cfg(target_os = "windows")]
fn get_exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn get_exe_dir() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "windows")]
fn windows_appdata_path(name: &str) -> String {
    std::env::var("APPDATA")
        .map(|appdata| format!("{}\\lianpkg\\{}", appdata, name))
        .unwrap_or_else(|_| format!(".\\{}", name))
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn windows_appdata_path(_name: &str) -> String {
    unreachable!()
}

/// 获取 Steam Workshop 路径实现
fn get_workshop_path_impl() -> String {
    const WALLPAPER_ENGINE_APP_ID: &str = "431960";

    if let Some(base_path) = get_steam_base_path() {
        if let Some(lib_path) = find_library_path(&base_path) {
            return lib_path
                .join("steamapps")
                .join("workshop")
                .join("content")
                .join(WALLPAPER_ENGINE_APP_ID)
                .display()
                .to_string();
        }
        return base_path
            .join("steamapps")
            .join("workshop")
            .join("content")
            .join(WALLPAPER_ENGINE_APP_ID)
            .display()
            .to_string();
    }

    #[cfg(target_os = "windows")]
    {
        r"C:\Program Files (x86)\Steam\steamapps\workshop\content\431960".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "~/.local/share/Steam/steamapps/workshop/content/431960".to_string()
    }
}

fn get_steam_base_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        get_steam_path_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        get_steam_path_linux()
    }
}

#[cfg(target_os = "windows")]
fn get_steam_path_windows() -> Option<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey("Software\\Valve\\Steam")
        .ok()
        .and_then(|steam| steam.get_value::<String, _>("SteamPath").ok())
        .map(PathBuf::from)
}

#[cfg(not(target_os = "windows"))]
fn get_steam_path_linux() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let candidates = [
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/data/Steam"),
        home.join("snap/steam/common/.steam/steam"),
        home.join(".steam/steam"),
    ];

    for path in &candidates {
        if path.exists() {
            return Some(path.clone());
        }
    }
    None
}

fn find_library_path(steam_base: &std::path::Path) -> Option<PathBuf> {
    let vdf_path = steam_base.join("steamapps").join("libraryfolders.vdf");
    if !vdf_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&vdf_path).ok()?;

    // 简单解析 VDF 格式查找包含 431960 的库
    for line in content.lines() {
        if line.contains("\"path\"") {
            if let Some(start) = line.find('"') {
                let rest = &line[start + 1..];
                if let Some(end) = rest.find('"') {
                    let path_str = &rest[..end];
                    // 跳过 "path" 本身，取下一个值
                    if path_str != "path" {
                        continue;
                    }
                }
            }
        }
    }

    // 简化处理：返回第一个有效库路径
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"path\"") {
            if let Some(start) = trimmed.rfind('"') {
                let before = &trimmed[..start];
                if let Some(second_last) = before.rfind('"') {
                    let path_str = &before[second_last + 1..];
                    let lib_path = PathBuf::from(path_str);
                    if lib_path.exists() {
                        return Some(lib_path);
                    }
                }
            }
        }
    }

    None
}
