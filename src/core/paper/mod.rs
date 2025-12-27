//! paper 模块 - Wallpaper 壁纸扫描与复制
//!
//! 支持两种使用模式：
//! - 单独使用：直接调用 extract_all 一键提取
//! - 复合流程：组合 list_dirs → check_pkg → process_folder 精细控制
//!
//! 主要接口：
//! - 扫描: list_dirs, read_meta, check_pkg, estimate
//! - 复制: process_folder, extract_all

mod structs;
mod scan;
mod copy;
mod utl;

// ============================================================================
// 导出配置结构体
// ============================================================================
pub use structs::PaperConfig;

// ============================================================================
// 导出 Input/Output 结构体
// ============================================================================

// 扫描相关
pub use structs::ListDirsInput;
pub use structs::ListDirsOutput;
pub use structs::ReadMetaInput;
pub use structs::ReadMetaOutput;
pub use structs::CheckPkgInput;
pub use structs::CheckPkgOutput;
pub use structs::EstimateInput;
pub use structs::EstimateOutput;

// 复制相关
pub use structs::ProcessFolderInput;
pub use structs::ProcessFolderOutput;
pub use structs::ExtractInput;
pub use structs::ExtractOutput;

// ============================================================================
// 导出运行时结构体
// ============================================================================
pub use structs::ProjectMeta;
pub use structs::WallpaperStats;
pub use structs::ProcessedFolder;
pub use structs::ProcessResultType;

// ============================================================================
// 导出扫描接口
// ============================================================================
pub use scan::list_dirs;
pub use scan::read_meta;
pub use scan::check_pkg;
pub use scan::estimate;

// ============================================================================
// 导出复制接口
// ============================================================================
pub use copy::process_folder;
pub use copy::extract_all;
