//! path 模块的 Input/Output 结构体定义
//!
//! 精简后的核心接口：
//! - ensure_dir: 确保目录存在
//! - expand_path: 展开 ~ 路径
//! - resolve_path: 统一路径解析
//! - scan_files: 扫描目标文件

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// ensure_dir - 确保目录存在
// ============================================================================

/// ensure_dir 接口入参
#[derive(Debug, Clone)]
pub struct EnsureDirInput {
    /// 目标目录路径
    pub path: PathBuf,
}

/// ensure_dir 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsureDirOutput {
    /// 目录路径
    pub path: PathBuf,
    /// 是否新创建（false 表示已存在）
    pub created: bool,
}

// ============================================================================
// expand_path - 展开 ~ 路径
// ============================================================================

/// expand_path 接口入参
#[derive(Debug, Clone)]
pub struct ExpandPathInput {
    /// 待展开的路径字符串（可能包含 ~）
    pub path: String,
}

/// expand_path 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandPathOutput {
    /// 展开后的完整路径
    pub path: PathBuf,
}

// ============================================================================
// scan_files - 扫描目标文件
// ============================================================================

/// scan_files 接口入参
#[derive(Debug, Clone)]
pub struct ScanFilesInput {
    /// 搜索路径（文件或目录）
    pub path: PathBuf,
    /// 文件扩展名过滤（可选，不填则默认 pkg/tex）
    pub extensions: Option<Vec<String>>,
}

/// scan_files 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFilesOutput {
    /// 目标文件列表
    pub files: Vec<PathBuf>,
}
