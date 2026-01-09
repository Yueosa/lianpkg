//! 统一错误处理模块
//!
//! 提供 core 模块的统一错误类型和 Result 别名。
//! 所有 core 子模块的接口函数都应返回 `CoreResult<T>`。

use serde::{Deserialize, Serialize};
use std::fmt;

/// 核心错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreError {
    /// 文件/目录 I/O 错误
    Io {
        message: String,
        path: Option<String>,
    },
    /// 解析错误 (TOML/JSON/PKG/TEX)
    Parse {
        message: String,
        source: Option<String>,
    },
    /// 验证错误 (参数无效、格式不符等)
    Validation { message: String },
    /// 未找到 (文件/目录/条目不存在)
    NotFound {
        message: String,
        path: Option<String>,
    },
    /// 格式不支持 (未知的文件格式等)
    Unsupported { message: String },
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Io { message, path } => {
                if let Some(p) = path {
                    write!(f, "IO error at '{}': {}", p, message)
                } else {
                    write!(f, "IO error: {}", message)
                }
            }
            CoreError::Parse { message, source } => {
                if let Some(s) = source {
                    write!(f, "Parse error in '{}': {}", s, message)
                } else {
                    write!(f, "Parse error: {}", message)
                }
            }
            CoreError::Validation { message } => {
                write!(f, "Validation error: {}", message)
            }
            CoreError::NotFound { message, path } => {
                if let Some(p) = path {
                    write!(f, "Not found '{}': {}", p, message)
                } else {
                    write!(f, "Not found: {}", message)
                }
            }
            CoreError::Unsupported { message } => {
                write!(f, "Unsupported: {}", message)
            }
        }
    }
}

impl std::error::Error for CoreError {}

/// 统一 Result 类型别名
pub type CoreResult<T> = Result<T, CoreError>;

// ============================================================================
// 便捷构造函数
// ============================================================================

impl CoreError {
    /// 创建 IO 错误
    pub fn io(msg: impl Into<String>) -> Self {
        CoreError::Io {
            message: msg.into(),
            path: None,
        }
    }

    /// 创建带路径的 IO 错误
    pub fn io_with_path(msg: impl Into<String>, path: impl Into<String>) -> Self {
        CoreError::Io {
            message: msg.into(),
            path: Some(path.into()),
        }
    }

    /// 创建解析错误
    pub fn parse(msg: impl Into<String>) -> Self {
        CoreError::Parse {
            message: msg.into(),
            source: None,
        }
    }

    /// 创建带来源的解析错误
    pub fn parse_with_source(msg: impl Into<String>, source: impl Into<String>) -> Self {
        CoreError::Parse {
            message: msg.into(),
            source: Some(source.into()),
        }
    }

    /// 创建验证错误
    pub fn validation(msg: impl Into<String>) -> Self {
        CoreError::Validation {
            message: msg.into(),
        }
    }

    /// 创建未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        CoreError::NotFound {
            message: msg.into(),
            path: None,
        }
    }

    /// 创建带路径的未找到错误
    pub fn not_found_with_path(msg: impl Into<String>, path: impl Into<String>) -> Self {
        CoreError::NotFound {
            message: msg.into(),
            path: Some(path.into()),
        }
    }

    /// 创建不支持错误
    pub fn unsupported(msg: impl Into<String>) -> Self {
        CoreError::Unsupported {
            message: msg.into(),
        }
    }
}

// ============================================================================
// From 实现 - 方便从标准库错误转换
// ============================================================================

impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        CoreError::Io {
            message: err.to_string(),
            path: None,
        }
    }
}

impl From<toml::de::Error> for CoreError {
    fn from(err: toml::de::Error) -> Self {
        CoreError::Parse {
            message: err.to_string(),
            source: Some("TOML".to_string()),
        }
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(err: serde_json::Error) -> Self {
        CoreError::Parse {
            message: err.to_string(),
            source: Some("JSON".to_string()),
        }
    }
}
