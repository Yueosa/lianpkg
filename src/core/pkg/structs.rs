//! 结构体定义 - Input/Output、运行时结构体

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

// ============================================================================
// Input 结构体
// ============================================================================

/// parse_pkg 接口入参
#[derive(Debug, Clone)]
pub struct ParsePkgInput {
    /// pkg 文件路径
    pub file_path: PathBuf,
}

/// unpack_pkg 接口入参
#[derive(Debug, Clone)]
pub struct UnpackPkgInput {
    /// pkg 文件路径
    pub file_path: PathBuf,
    /// 输出目录
    pub output_base: PathBuf,
}

/// unpack_entry 接口入参
#[derive(Debug, Clone)]
pub struct UnpackEntryInput {
    /// pkg 文件原始数据
    pub pkg_data: Vec<u8>,
    /// 数据区起始偏移
    pub data_start: usize,
    /// 要解包的条目
    pub entry: PkgEntry,
    /// 输出路径
    pub output_path: PathBuf,
}

// ============================================================================
// Output 结构体
// ============================================================================

/// parse_pkg 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsePkgOutput {
    /// 是否成功
    pub success: bool,
    /// pkg 文件信息，失败时为 None
    pub pkg_info: Option<PkgInfo>,
    /// 错误信息，成功时为 None
    pub error: Option<String>,
}

/// unpack_pkg 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnpackPkgOutput {
    /// 是否成功
    pub success: bool,
    /// pkg 文件信息
    pub pkg_info: Option<PkgInfo>,
    /// 解包的文件列表
    pub extracted_files: Vec<ExtractedFile>,
    /// 错误信息，成功时为 None
    pub error: Option<String>,
}

/// unpack_entry 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnpackEntryOutput {
    /// 是否成功
    pub success: bool,
    /// 输出路径
    pub output_path: PathBuf,
    /// 错误信息，成功时为 None
    pub error: Option<String>,
}

// ============================================================================
// 运行时结构体
// ============================================================================

/// Pkg 文件信息（解析结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgInfo {
    /// pkg 版本字符串
    pub version: String,
    /// 文件数量
    pub file_count: u32,
    /// 文件条目列表
    pub entries: Vec<PkgEntry>,
    /// 数据区起始偏移
    pub data_start: usize,
}

/// 文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgEntry {
    /// 文件名
    pub name: String,
    /// 在数据区中的偏移
    pub offset: u32,
    /// 文件大小
    pub size: u32,
}

/// 解包后的文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFile {
    /// 原始条目名
    pub entry_name: String,
    /// 输出路径
    pub output_path: PathBuf,
    /// 文件大小
    pub size: u32,
}
