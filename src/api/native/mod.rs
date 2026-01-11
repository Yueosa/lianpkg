//! native 模块 - 原生 API 层
//!
//! 提供 CLI 和 GUI (Flutter) 调用的统一接口层。
//! 封装 core 模块的底层操作，提供更友好的高级 API。
//!
//! ## 模块结构
//!
//! - `cfg`: 配置管理（初始化、加载、保存）
//! - `paper`: 壁纸处理（扫描、复制）
//! - `pkg`: PKG 处理（预览、解包）
//! - `tex`: TEX 处理（预览、转换）
//! - `pipeline`: 流水线执行（完整流程）
//!
//! ## 使用示例
//!
//! ### 快速执行完整流水线
//! ```rust,ignore
//! use lianpkg::api::native::pipeline;
//!
//! let result = pipeline::quick_run(pipeline::QuickRunInput {
//!     config_dir: None,  // 使用默认配置目录
//!     force_all: false,  // 增量处理
//! });
//!
//! if result.success {
//!     println!("处理了 {} 个壁纸", result.stats.wallpapers_processed);
//! }
//! ```
//!
//! ### 分步执行
//! ```rust,ignore
//! use lianpkg::api::native::{cfg, paper, pkg, tex};
//!
//! // 1. 初始化配置
//! let init = cfg::init_config(cfg::InitConfigInput { config_dir: None });
//!
//! // 2. 加载配置
//! let config = cfg::load_config(cfg::LoadConfigInput {
//!     config_path: init.config_path,
//! }).config.unwrap();
//!
//! // 3. 扫描壁纸
//! let wallpapers = paper::scan_wallpapers(paper::ScanWallpapersInput {
//!     workshop_path: config.workshop_path.clone(),
//! });
//!
//! // 4. 复制壁纸
//! let copied = paper::copy_wallpapers(paper::CopyWallpapersInput {
//!     wallpaper_ids: None,
//!     workshop_path: config.workshop_path,
//!     raw_output_path: config.raw_output_path,
//!     pkg_temp_path: config.pkg_temp_path.clone(),
//!     enable_raw: config.enable_raw_output,
//! });
//!
//! // 5. 解包 PKG
//! let unpacked = pkg::unpack_all(pkg::UnpackAllInput {
//!     pkg_temp_path: config.pkg_temp_path,
//!     unpacked_output_path: config.unpacked_output_path.clone(),
//! });
//!
//! // 6. 转换 TEX
//! let converted = tex::convert_all(tex::ConvertAllInput {
//!     unpacked_path: config.unpacked_output_path,
//!     output_path: config.converted_output_path,
//! });
//! ```

pub mod cfg;
pub mod paper;
pub mod pipeline;
pub mod pkg;
pub mod tex;

// ============================================================================
// 导出配置模块
// ============================================================================
pub use cfg::{
    add_processed_wallpaper,
    // 接口
    init_config,
    is_wallpaper_processed,
    load_config,
    load_state,
    save_state,
    update_statistics,
    // 结构体
    InitConfigInput,
    InitConfigOutput,
    LoadConfigInput,
    LoadConfigOutput,
    LoadStateInput,
    LoadStateOutput,
    PipelineConfig,
    RuntimeConfig,
    SaveStateInput,
    SaveStateOutput,
};

// ============================================================================
// 导出壁纸模块
// ============================================================================
pub use paper::{
    copy_wallpapers,
    get_wallpaper_detail,
    // 接口
    scan_wallpapers,
    CopyResult,
    CopyResultType,
    CopyStats,
    CopyWallpapersInput,
    CopyWallpapersOutput,
    ScanStats,
    // 结构体
    ScanWallpapersInput,
    ScanWallpapersOutput,
    WallpaperInfo,
};

// ============================================================================
// 导出 PKG 模块
// ============================================================================
pub use pkg::{
    get_tex_files_from_unpacked,
    preview_pkg,
    // 接口
    unpack_all,
    unpack_single,
    PkgFileEntry,
    PkgPreview,
    PreviewPkgInput,
    PreviewPkgOutput,
    // 结构体
    UnpackAllInput,
    UnpackAllOutput,
    UnpackResult,
    UnpackStats,
    UnpackedFile,
};

// ============================================================================
// 导出 TEX 模块
// ============================================================================
pub use tex::{
    // 接口
    convert_all,
    convert_single,
    preview_tex,
    // 结构体
    ConvertAllInput,
    ConvertAllOutput,
    ConvertResult,
    ConvertStats,
    PreviewTexInput,
    PreviewTexOutput,
    TexPreview,
};

// ============================================================================
// 导出流水线模块
// ============================================================================
pub use pipeline::{
    clean_unpacked_dir,
    copy_metadata_to_tex_converted,
    estimate_disk_usage,
    quick_run,
    // 接口
    run_pipeline,
    run_pkg_only,
    run_tex_only,
    DebugLogCallback,
    DebugLogEvent,
    DebugLogType,
    EstimateDiskInput,
    EstimateDiskOutput,
    PipelineOverrides,
    PipelineProgress,
    PipelineStage,
    PipelineStats,
    // 回调类型
    ProgressCallback,
    QuickRunInput,
    QuickRunOutput,
    // 结构体
    RunPipelineInput,
    RunPipelineOutput,
};
