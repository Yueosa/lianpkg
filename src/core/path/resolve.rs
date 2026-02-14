//! 统一路径解析接口
//!
//! 将多个路径生成函数合并为单一 `resolve_path` 接口。
//! 提供 `detect_workshop_path()` 自动探测 Wallpaper Engine Workshop 路径。

use crate::core::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Wallpaper Engine 的 Steam App ID
const WALLPAPER_ENGINE_APP_ID: &str = "431960";

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
    /// Steam Workshop 路径（自动探测）
    Workshop,
    /// 原始壁纸输出路径
    RawOutput,
    /// 解包输出路径
    UnpackedOutput,
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
    /// 解析后的路径
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
        PathType::UnpackedOutput => resolve_unpacked_output(),
        PathType::SceneName { stem } => resolve_scene_name(&stem),
        PathType::TexOutput {
            tex_path,
            output_base,
        } => resolve_tex_output(&tex_path, &output_base),
    }
}

// ============================================================================
// 内部路径解析
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
    let path = detect_workshop_path()?;
    let path_str = path.display().to_string();
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_raw_output() -> CoreResult<ResolvePathOutput> {
    let path = {
        #[cfg(target_os = "windows")]
        {
            windows_data_path("Wallpapers_Raw")
        }
        #[cfg(not(target_os = "windows"))]
        {
            expand_tilde("~/.local/share/lianpkg/Wallpapers_Raw")
        }
    };
    let path_str = path.display().to_string();
    Ok(ResolvePathOutput { path, path_str })
}

fn resolve_unpacked_output() -> CoreResult<ResolvePathOutput> {
    let path = {
        #[cfg(target_os = "windows")]
        {
            windows_data_path("Pkg_Unpacked")
        }
        #[cfg(not(target_os = "windows"))]
        {
            expand_tilde("~/.local/share/lianpkg/Pkg_Unpacked")
        }
    };
    let path_str = path.display().to_string();
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
    tex_path: &Path,
    output_base: &Path,
) -> CoreResult<ResolvePathOutput> {
    let base_dir = output_base
        .join(tex_path.file_stem().unwrap_or_default())
        .join("tex_converted");

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
// Workshop 路径探测
// ============================================================================

/// 自动探测 Wallpaper Engine Workshop 内容路径
///
/// 遍历所有 Steam library folder，找到包含
/// `steamapps/workshop/content/431960` 的那个并返回完整路径。
///
/// 探测流程：
/// 1. 收集所有 Steam 安装候选基路径（Linux 最多 4 个，Windows 从注册表读取）
/// 2. 对每个基路径解析 `steamapps/libraryfolders.vdf`，提取所有 library path
/// 3. 将基路径本身也加入候选列表（VDF 可能不包含自身）
/// 4. 逐个检查 `{library}/steamapps/workshop/content/431960/` 是否存在
/// 5. 返回第一个命中；全部未命中则返回 `CoreError::NotFound`
pub fn detect_workshop_path() -> CoreResult<PathBuf> {
    let mut all_library_paths: Vec<PathBuf> = Vec::new();

    for base in get_steam_base_candidates() {
        // 基路径自身也是候选（VDF 可能不包含自身）
        all_library_paths.push(base.clone());
        // 从 VDF 中提取其他 library path
        all_library_paths.extend(parse_library_folders_vdf(&base));
    }

    // 去重（保持顺序）
    let mut seen = std::collections::HashSet::new();
    all_library_paths.retain(|p| seen.insert(p.clone()));

    // 逐个检查 workshop/content/431960 是否存在
    for lib in &all_library_paths {
        let workshop = lib
            .join("steamapps")
            .join("workshop")
            .join("content")
            .join(WALLPAPER_ENGINE_APP_ID);
        if workshop.is_dir() {
            return Ok(workshop);
        }
    }

    Err(CoreError::not_found(
        "未找到 Wallpaper Engine Workshop 路径，请在 config.toml 中手动设置 workshop_path",
    ))
}

// ============================================================================
// Steam 路径候选
// ============================================================================

/// 获取所有可能的 Steam 安装基路径
fn get_steam_base_candidates() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        get_steam_candidates_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        get_steam_candidates_linux()
    }
}

#[cfg(target_os = "windows")]
fn get_steam_candidates_windows() -> Vec<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;

    let mut candidates = Vec::new();

    // 从注册表读取
    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
        if let Ok(path_str) = hkcu.get_value::<String, _>("SteamPath") {
            let p = PathBuf::from(path_str);
            if p.exists() {
                candidates.push(p);
            }
        }
    }

    // 常见默认路径作为 fallback
    let default = PathBuf::from(r"C:\Program Files (x86)\Steam");
    if default.exists() && !candidates.contains(&default) {
        candidates.push(default);
    }

    candidates
}

#[cfg(not(target_os = "windows"))]
fn get_steam_candidates_linux() -> Vec<PathBuf> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    let candidates = [
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
        home.join(".var/app/com.valvesoftware.Steam/data/Steam"),
        home.join("snap/steam/common/.steam/steam"),
    ];

    candidates
        .into_iter()
        .filter(|p| p.exists())
        .collect()
}

// ============================================================================
// VDF 解析
// ============================================================================

/// 解析 `libraryfolders.vdf`，提取所有 library path
///
/// VDF 格式示例：
/// ```text
/// "libraryfolders"
/// {
///     "0"
///     {
///         "path"		"/home/user/.local/share/Steam"
///     }
///     "1"
///     {
///         "path"		"/mnt/games/SteamLibrary"
///     }
/// }
/// ```
fn parse_library_folders_vdf(steam_base: &Path) -> Vec<PathBuf> {
    let vdf_path = steam_base.join("steamapps").join("libraryfolders.vdf");

    let content = match std::fs::read_to_string(&vdf_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut paths = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        // 匹配形如 "path"		"/some/path" 的行
        if !trimmed.starts_with("\"path\"") {
            continue;
        }

        // 提取值：找最后一对引号之间的内容
        // "path"		"/mnt/games/SteamLibrary"
        //              ^                       ^
        if let Some(value) = extract_vdf_value(trimmed) {
            // Windows VDF 中路径使用 \\\\ 转义
            let cleaned = value.replace("\\\\", "\\");
            let lib_path = PathBuf::from(&cleaned);
            if lib_path.exists() {
                paths.push(lib_path);
            }
        }
    }

    paths
}

/// 从 VDF 行中提取值（最后一对引号之间的内容）
///
/// 输入: `"path"		"/mnt/games/SteamLibrary"`
/// 输出: `Some("/mnt/games/SteamLibrary")`
fn extract_vdf_value(line: &str) -> Option<&str> {
    // 找到最后一个引号
    let last_quote = line.rfind('"')?;
    // 在其之前找倒数第二个引号
    let before = &line[..last_quote];
    let second_last_quote = before.rfind('"')?;
    let value = &line[second_last_quote + 1..last_quote];
    // 排除 key 本身（"path"）
    if value == "path" {
        return None;
    }
    Some(value)
}

// ============================================================================
// 平台辅助
// ============================================================================

/// 展开路径中的 `~` 为实际 home 目录
pub(crate) fn expand_tilde(raw: &str) -> PathBuf {
    if raw.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&raw[2..]);
        }
    } else if raw == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(raw)
}

/// Windows: 获取数据目录下的子路径
#[cfg(target_os = "windows")]
fn windows_data_path(name: &str) -> PathBuf {
    // 优先 exe 同级目录, 其次 APPDATA
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join(name);
        }
    }
    std::env::var("APPDATA")
        .map(|appdata| PathBuf::from(appdata).join("lianpkg").join(name))
        .unwrap_or_else(|_| PathBuf::from(name))
}
