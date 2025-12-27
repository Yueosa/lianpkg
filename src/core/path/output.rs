//! 输出路径解析
//!
//! 提供各类输出目录的默认路径

#[cfg(target_os = "windows")]
use std::env;

/// 获取默认的原始壁纸输出路径
/// - Windows: %APPDATA%/lianpkg/Wallpapers_Raw
/// - Linux: ~/.local/share/lianpkg/Wallpapers_Raw
pub fn default_raw_output_path() -> String {
    #[cfg(target_os = "windows")]
    {
        windows_appdata_path("Wallpapers_Raw")
    }
    #[cfg(not(target_os = "windows"))]
    {
        "~/.local/share/lianpkg/Wallpapers_Raw".to_string()
    }
}

/// 获取默认的 Pkg 临时路径
/// - Windows: %APPDATA%/lianpkg/Pkg_Temp
/// - Linux: ~/.local/share/lianpkg/Pkg_Temp
pub fn default_pkg_temp_path() -> String {
    #[cfg(target_os = "windows")]
    {
        windows_appdata_path("Pkg_Temp")
    }
    #[cfg(not(target_os = "windows"))]
    {
        "~/.local/share/lianpkg/Pkg_Temp".to_string()
    }
}

/// 获取默认的解包输出路径
/// - Windows: %APPDATA%/lianpkg/Pkg_Unpacked
/// - Linux: ~/.local/share/lianpkg/Pkg_Unpacked
pub fn default_unpacked_output_path() -> String {
    #[cfg(target_os = "windows")]
    {
        windows_appdata_path("Pkg_Unpacked")
    }
    #[cfg(not(target_os = "windows"))]
    {
        "~/.local/share/lianpkg/Pkg_Unpacked".to_string()
    }
}

/// Windows: 获取 AppData 下的子路径
#[cfg(target_os = "windows")]
fn windows_appdata_path(sub: &str) -> String {
    use std::path::PathBuf;
    
    env::var("APPDATA")
        .map(|p| PathBuf::from(p).join("lianpkg").join(sub).to_string_lossy().to_string())
        .unwrap_or_else(|_| format!(".\\{}", sub))
}
