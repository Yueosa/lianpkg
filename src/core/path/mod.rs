//! path 模块 - 路径处理与解析
//!
//! ## 核心接口
//!
//! | 接口 | 功能 |
//! |------|------|
//! | `ensure_dir` | 确保目录存在，不存在则递归创建 |
//! | `expand_path` | 展开路径中的 `~` 为用户主目录 |
//! | `resolve_path` | 统一路径解析（配置、输出、Workshop 等） |
//! | `detect_workshop_path` | 自动探测 Wallpaper Engine Workshop 路径 |
//! | `scan_files` | 扫描目标文件（递归，支持扩展名过滤） |
//!
//! ## 路径类型 (PathType)
//!
//! `resolve_path` 通过 `PathType` 枚举支持多种路径解析：
//! - `ConfigDir` - 配置目录
//! - `ConfigToml` - config.toml 路径
//! - `StateJson` - state.json 路径
//! - `Workshop` - Steam Workshop 路径（自动探测）
//! - `RawOutput` - 原始壁纸输出路径
//! - `UnpackedOutput` - 解包输出路径
//! - `SceneName { stem }` - 从 PKG stem 提取场景名
//! - `TexOutput { tex_path, output_base }` - TEX 输出目录

mod resolve;
mod scan;
mod types;
mod utl;

// ============================================================================
// 导出 Input/Output 结构体
// ============================================================================
pub use types::EnsureDirInput;
pub use types::EnsureDirOutput;
pub use types::ExpandPathInput;
pub use types::ExpandPathOutput;
pub use types::ScanFilesInput;
pub use types::ScanFilesOutput;

// ============================================================================
// 导出 resolve_path 相关
// ============================================================================
pub use resolve::detect_workshop_path;
pub use resolve::resolve_path;
pub use resolve::PathType;
pub use resolve::ResolvePathInput;
pub use resolve::ResolvePathOutput;

// ============================================================================
// 导出核心接口
// ============================================================================
pub use scan::scan_files;
pub use utl::ensure_dir;
pub use utl::expand_path;

// ============================================================================
// 兼容层（供 api/native 和 cli 过渡使用）
// 这些函数将在 api/cli 迁移到新接口后移除
// ============================================================================

use std::path::{Path, PathBuf};

/// 兼容层：展开路径中的 ~
pub fn expand_path_compat(path_str: &str) -> PathBuf {
    resolve::expand_tilde(path_str)
}

/// 兼容层：确保目录存在
pub fn ensure_dir_compat(path: &Path) -> Result<(), String> {
    ensure_dir(EnsureDirInput {
        path: path.to_path_buf(),
    })
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// 兼容层：获取默认配置目录
pub fn default_config_dir() -> PathBuf {
    resolve_path(ResolvePathInput {
        path_type: PathType::ConfigDir,
    })
    .map(|o| o.path)
    .unwrap_or_else(|_| PathBuf::from("."))
}

/// 兼容层：获取默认原始壁纸输出路径
pub fn default_raw_output_path() -> String {
    resolve_path(ResolvePathInput {
        path_type: PathType::RawOutput,
    })
    .map(|o| o.path_str)
    .unwrap_or_else(|_| "~/.local/share/lianpkg/Wallpapers_Raw".to_string())
}

/// 内部：获取 Pkg_Unpacked 基础路径
fn pkg_base_path() -> String {
    resolve_path(ResolvePathInput {
        path_type: PathType::UnpackedOutput,
    })
    .map(|o| o.path_str)
    .unwrap_or_else(|_| "~/.local/share/lianpkg/Pkg_Unpacked".to_string())
}

/// 兼容层：获取默认解包输出路径（Pkg_Unpacked/unpacked）
pub fn default_unpacked_output_path() -> String {
    let p = std::path::PathBuf::from(pkg_base_path()).join("unpacked");
    p.display().to_string()
}

/// 兼容层：获取默认 TEX 转换输出路径（Pkg_Unpacked/tex_converted）
pub fn default_converted_output_path() -> String {
    let p = std::path::PathBuf::from(pkg_base_path()).join("tex_converted");
    p.display().to_string()
}

/// 兼容层：从 pkg 文件名提取场景名
pub fn scene_name_from_pkg_stem(stem: &str) -> String {
    resolve_path(ResolvePathInput {
        path_type: PathType::SceneName {
            stem: stem.to_string(),
        },
    })
    .map(|o| o.path_str)
    .unwrap_or_else(|_| stem.to_string())
}

/// 兼容层：解析 tex 输出目录
pub fn resolve_tex_output_dir_compat(
    converted_output_path: Option<&str>,
    scene_root: &Path,
    input_file: Option<&Path>,
    _relative_base: Option<&Path>,
) -> PathBuf {
    let output_base = if let Some(custom) = converted_output_path {
        expand_path_compat(custom)
    } else {
        scene_root.to_path_buf()
    };

    let tex_path = input_file.unwrap_or(scene_root);

    resolve_path(ResolvePathInput {
        path_type: PathType::TexOutput {
            tex_path: tex_path.to_path_buf(),
            output_base,
        },
    })
    .map(|o| o.path)
    .unwrap_or_else(|_| scene_root.join("tex_converted"))
}
