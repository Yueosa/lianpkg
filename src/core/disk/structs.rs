//! 磁盘模块结构体定义

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Input 结构体
// ============================================================================

/// 检查磁盘空间入参
#[derive(Debug, Clone)]
pub struct CheckSpaceInput {
    /// 要检查的路径（如果不存在会查找父目录）
    pub path: PathBuf,
}

// ============================================================================
// Output 结构体
// ============================================================================

/// 检查磁盘空间返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckSpaceOutput {
    /// 可用空间（字节）
    pub available: u64,
    /// 总空间（字节）
    pub total: u64,
    /// 实际检查的路径（可能是输入路径的父目录）
    pub check_path: PathBuf,
}
