//! path 模块 - 路径处理与解析
//!
//! 本模块提供各类路径解析功能：
//! - 配置文件路径: default_config_dir, default_config_toml_path, default_state_json_path
//! - Steam/Workshop 路径: default_workshop_path
//! - 输出路径: default_raw_output_path, default_pkg_temp_path, default_unpacked_output_path
//! - Pkg 路径: pkg_temp_dest, scene_name_from_pkg_stem
//! - Tex 路径: resolve_tex_output_dir
//! - 文件扫描: get_target_files, find_project_root
//! - 通用工具: ensure_dir, expand_path, get_unique_output_path

mod utl;    // 通用工具函数
mod cfg;    // Config 路径解析
mod steam;  // Steam/Wallpaper 路径定位
mod output; // 输出路径解析
mod pkg;    // Pkg 路径解析
mod tex;    // Tex 路径解析
mod scan;   // 文件扫描相关

// ============================================================================
// 导出通用工具函数
// ============================================================================
pub use utl::ensure_dir;
pub use utl::expand_path;
pub use utl::get_unique_output_path;

// ============================================================================
// 导出 Config 路径接口
// ============================================================================
pub use cfg::default_config_dir;
pub use cfg::default_config_toml_path;
pub use cfg::default_state_json_path;
pub use cfg::exe_dir;
pub use cfg::exe_config_dir;

// ============================================================================
// 导出 Steam/Workshop 路径接口
// ============================================================================
pub use steam::default_workshop_path;

// ============================================================================
// 导出输出路径接口
// ============================================================================
pub use output::default_raw_output_path;
pub use output::default_pkg_temp_path;
pub use output::default_unpacked_output_path;

// ============================================================================
// 导出 Pkg 路径接口
// ============================================================================
pub use pkg::pkg_temp_dest;
pub use pkg::scene_name_from_pkg_stem;

// ============================================================================
// 导出 Tex 路径接口
// ============================================================================
pub use tex::resolve_tex_output_dir;

// ============================================================================
// 导出文件扫描接口
// ============================================================================
pub use scan::get_target_files;
pub use scan::find_project_root;

