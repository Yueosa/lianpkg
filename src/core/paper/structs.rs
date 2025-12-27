//! 结构体定义 - 配置、Input/Output、运行时结构体

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

// ============================================================================
// 配置结构体
// ============================================================================

/// Paper 模块运行配置
/// 可用于单独运行，也可嵌入复合流程
#[derive(Debug, Clone)]
pub struct PaperConfig {
    /// workshop 搜索路径
    pub search_path: PathBuf,
    /// 原始壁纸输出路径
    pub raw_output: PathBuf,
    /// pkg 临时输出路径
    pub pkg_temp_output: PathBuf,
    /// 是否提取原始壁纸
    pub enable_raw: bool,
}

// ============================================================================
// Input 结构体
// ============================================================================

/// list_dirs 接口入参
#[derive(Debug, Clone)]
pub struct ListDirsInput {
    /// 搜索路径
    pub path: PathBuf,
}

/// read_meta 接口入参
#[derive(Debug, Clone)]
pub struct ReadMetaInput {
    /// 壁纸文件夹路径
    pub folder: PathBuf,
}

/// check_pkg 接口入参
#[derive(Debug, Clone)]
pub struct CheckPkgInput {
    /// 壁纸文件夹路径
    pub folder: PathBuf,
}

/// estimate 接口入参
#[derive(Debug, Clone)]
pub struct EstimateInput {
    /// 搜索路径
    pub search_path: PathBuf,
    /// 是否计算原始壁纸大小
    pub enable_raw: bool,
}

/// process_folder 接口入参
#[derive(Debug, Clone)]
pub struct ProcessFolderInput {
    /// 壁纸文件夹路径
    pub folder: PathBuf,
    /// 原始壁纸输出路径
    pub raw_output: PathBuf,
    /// pkg 临时输出路径
    pub pkg_temp_output: PathBuf,
    /// 是否提取原始壁纸
    pub enable_raw: bool,
}

/// extract_all 接口入参
#[derive(Debug, Clone)]
pub struct ExtractInput {
    /// 运行配置
    pub config: PaperConfig,
}

// ============================================================================
// Output 结构体
// ============================================================================

/// list_dirs 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDirsOutput {
    /// 是否成功
    pub success: bool,
    /// 目录列表
    pub dirs: Vec<String>,
}

/// read_meta 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadMetaOutput {
    /// 是否成功
    pub success: bool,
    /// 元数据，失败时为 None
    pub meta: Option<ProjectMeta>,
}

/// check_pkg 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckPkgOutput {
    /// 是否包含 pkg 文件
    pub has_pkg: bool,
    /// pkg 文件列表
    pub pkg_files: Vec<PathBuf>,
}

/// estimate 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateOutput {
    /// pkg 文件总大小（字节）
    pub pkg_size: u64,
    /// 原始壁纸总大小（字节）
    pub raw_size: u64,
    /// pkg 壁纸数量
    pub pkg_count: usize,
    /// 原始壁纸数量
    pub raw_count: usize,
}

/// process_folder 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessFolderOutput {
    /// 是否复制了原始壁纸
    pub copied_raw: bool,
    /// 复制的 pkg 文件数量
    pub copied_pkgs: usize,
    /// 是否跳过（已存在等原因）
    pub skipped: bool,
    /// 处理结果类型
    pub result_type: ProcessResultType,
    /// 复制的 pkg 文件路径列表
    pub pkg_files: Vec<PathBuf>,
}

/// extract_all 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractOutput {
    /// 统计信息
    pub stats: WallpaperStats,
    /// 处理的文件夹详情列表
    pub processed_folders: Vec<ProcessedFolder>,
}

// ============================================================================
// 运行时结构体
// ============================================================================

/// project.json 元数据
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProjectMeta {
    pub contentrating: Option<String>,
    pub description: Option<String>,
    pub file: Option<String>,
    pub preview: Option<String>,
    pub tags: Option<Vec<String>>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub wallpaper_type: Option<String>,
    pub version: Option<u32>,
    pub workshopid: Option<String>,
    pub workshopurl: Option<String>,
    #[serde(default)]
    pub general: Option<serde_json::Value>,
}

/// 壁纸统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WallpaperStats {
    /// 原始壁纸数量
    pub raw_count: usize,
    /// pkg 壁纸数量
    pub pkg_count: usize,
    /// 总处理大小（字节）
    pub total_size: u64,
}

/// 处理结果详情（用于复合流程传递）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedFolder {
    /// 文件夹名称
    pub folder_name: String,
    /// 文件夹路径
    pub folder_path: PathBuf,
    /// 处理结果类型
    pub result_type: ProcessResultType,
    /// 复制的 pkg 文件列表
    pub pkg_files: Vec<PathBuf>,
}

/// 处理结果类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessResultType {
    /// 复制为原始壁纸
    Raw,
    /// 复制了 pkg 文件
    Pkg,
    /// 跳过（已存在等）
    Skipped,
}

