//! Steam/Wallpaper 路径定位
//!
//! 支持多种 Steam 安装方式：
//! - Windows: 通过注册表定位
//! - Linux 原生安装: ~/.local/share/Steam
//! - Linux Flatpak: ~/.var/app/com.valvesoftware.Steam/data/Steam
//! - Linux Snap: ~/snap/steam/common/.steam/steam
//! - Linux 软链接: ~/.steam/steam

use std::fs;
use std::path::{Path, PathBuf};

/// Wallpaper Engine 的 Steam App ID
const WALLPAPER_ENGINE_APP_ID: &str = "431960";

/// 获取默认的 Workshop 路径
/// 优先尝试定位实际安装位置，失败则返回平台默认值
pub fn default_workshop_path() -> String {
    if let Some(base_path) = get_steam_base_path() {
        // 尝试从 libraryfolders.vdf 查找实际库路径
        if let Some(lib_path) = find_library_path(&base_path) {
            return lib_path
                .join("steamapps")
                .join("workshop")
                .join("content")
                .join(WALLPAPER_ENGINE_APP_ID)
                .to_string_lossy()
                .to_string();
        }
        
        // 回退到默认 Steam 库
        return base_path
            .join("steamapps")
            .join("workshop")
            .join("content")
            .join(WALLPAPER_ENGINE_APP_ID)
            .to_string_lossy()
            .to_string();
    }

    // 未找到 Steam，返回平台默认路径
    #[cfg(target_os = "windows")]
    {
        r"C:\Program Files (x86)\Steam\steamapps\workshop\content\431960".to_string()
    }

    #[cfg(not(target_os = "windows"))]
    {
        "~/.local/share/Steam/steamapps/workshop/content/431960".to_string()
    }
}

/// 获取 Steam 基础安装路径
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

/// Windows: 通过注册表获取 Steam 路径
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

/// Linux: 按优先级检查多种安装方式
#[cfg(not(target_os = "windows"))]
fn get_steam_path_linux() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    
    // 候选路径列表（按优先级排序）
    let candidates = [
        // 1. 原生安装 (XDG_DATA_HOME)
        get_xdg_steam_path(),
        // 2. 原生安装 (默认位置)
        Some(home.join(".local/share/Steam")),
        // 3. Flatpak 安装
        Some(home.join(".var/app/com.valvesoftware.Steam/data/Steam")),
        // 4. Snap 安装
        Some(home.join("snap/steam/common/.steam/steam")),
        // 5. 旧版软链接位置
        Some(home.join(".steam/steam")),
    ];
    
    // 遍历候选路径，返回第一个有效的
    for candidate in candidates.into_iter().flatten() {
        if is_valid_steam_path(&candidate) {
            return Some(resolve_symlink(&candidate));
        }
    }
    
    None
}

/// 获取 XDG_DATA_HOME 下的 Steam 路径
#[cfg(not(target_os = "windows"))]
fn get_xdg_steam_path() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .map(|p| p.join("Steam"))
}

/// 检查路径是否为有效的 Steam 安装
#[cfg(not(target_os = "windows"))]
fn is_valid_steam_path(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    
    // 检查关键目录/文件是否存在
    let steamapps = path.join("steamapps");
    let steam_exe = path.join("steam.sh");
    
    // steamapps 目录必须存在
    if steamapps.exists() && steamapps.is_dir() {
        return true;
    }
    
    // 或者 steam.sh 存在
    if steam_exe.exists() {
        return true;
    }
    
    false
}

/// 解析软链接，返回真实路径
#[cfg(not(target_os = "windows"))]
fn resolve_symlink(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// 从 libraryfolders.vdf 解析 Steam 库路径
/// 查找包含 Wallpaper Engine (431960) 的库
fn find_library_path(steam_base: &Path) -> Option<PathBuf> {
    let vdf_path = steam_base.join("steamapps").join("libraryfolders.vdf");
    if !vdf_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&vdf_path).ok()?;
    let mut current_path: Option<PathBuf> = None;
    
    for line in content.lines() {
        let line = line.trim();
        
        // 匹配 "path" "..." 行
        if line.starts_with("\"path\"") {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 4 {
                let p = parts[3].replace("\\\\", "\\");
                current_path = Some(PathBuf::from(p));
            }
        }
        
        // 匹配 "431960" 行（Wallpaper Engine）
        if line.contains(&format!("\"{}\"", WALLPAPER_ENGINE_APP_ID)) {
            return current_path;
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_workshop_path_not_empty() {
        let path = default_workshop_path();
        assert!(!path.is_empty());
        assert!(path.contains("431960"));
    }
}
