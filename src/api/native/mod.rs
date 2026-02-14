//! native 模块 - 原生 API 层
//!
//! 提供 CLI 和 GUI (Flutter) 调用的统一接口层。
//! 封装 core 模块的底层操作，提供类型安全的高级 API。
//!
//! ## 模块结构
//!
//! - `context`: 应用上下文与配置管理（AppContext, init）
//! - `scan`: 壁纸扫描与复制
//! - `auto`: 自动批处理流水线
//! - `pkg`: PKG 解包与预览
//! - `tex`: TEX 转换与预览
//! - `util`: 内部工具函数
//!
//! ## 使用示例
//!
//! ### 快速执行完整流水线
//! ```rust,ignore
//! use lianpkg::api::native::{context, auto};
//!
//! let ctx = context::init(None)?;
//! let result = auto::run_auto(&ctx, auto::AutoOptions {
//!     wallpaper_ids: None,
//!     progress: None,
//! })?;
//! println!("处理了 {} 个壁纸", result.stats.wallpapers_processed);
//! ```
//!
//! ### 分步执行
//! ```rust,ignore
//! use lianpkg::api::native::{context, scan, pkg, tex};
//!
//! // 1. 初始化
//! let ctx = context::init(None)?;
//!
//! // 2. 扫描壁纸
//! let wallpapers = scan::scan(&ctx)?;
//!
//! // 3. 复制壁纸
//! let copied = scan::copy_wallpapers(
//!     &wallpapers.wallpapers,
//!     &ctx.config.raw_output_path,
//!     ctx.config.enable_raw_output,
//! )?;
//!
//! // 4. 解包 PKG
//! let pkg_sources: Vec<pkg::PkgSource> = copied.results.iter()
//!     .filter(|r| r.result_type == scan::CopyResultType::Pkg)
//!     .map(|r| pkg::PkgSource {
//!         wallpaper_id: r.wallpaper_id.clone(),
//!         pkg_paths: r.pkg_files.clone(),
//!     }).collect();
//! let unpacked = pkg::unpack_all(&pkg_sources, &ctx.config.unpacked_output_path)?;
//!
//! // 5. 转换 TEX
//! let converted = tex::convert_all(
//!     &ctx.config.unpacked_output_path,
//!     ctx.config.converted_output_path.as_deref(),
//! )?;
//! ```

pub mod context;
pub mod scan;
pub mod auto;
pub mod pkg;
pub mod tex;
pub(crate) mod util;

// ============================================================================
// 导出核心类型（便捷访问）
// ============================================================================
pub use context::{AppContext, RuntimeConfig, PipelineConfig, init};
pub use scan::{
    ScanOutput, WallpaperInfo, ScanStats,
    CopyOutput, CopyResult, CopyResultType, CopyStats,
};
pub use auto::{
    AutoOutput, AutoOptions, AutoProgress, AutoStage,
    PipelineStats, DiskEstimateOutput,
};
pub use pkg::{PkgSource, UnpackOutput, UnpackResult, UnpackStats, UnpackedFile, PkgPreview, PkgFileEntry};
pub use tex::{ConvertOutput, ConvertResult, ConvertStats, TexPreview};
