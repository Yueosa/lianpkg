//! 配置错误处理模块
//!
//! 本模块定义了配置系统中的**错误类型体系**和**验证结果结构**，
//! 提供统一的错误处理和验证反馈机制。
//!
//! ## 核心类型
//!
//! - [`ConfigError`]：配置操作过程中可能发生的所有错误类型
//! - [`ValidationResult`]：配置验证结果，包含错误和警告信息
//! - [`ValidationError`]：具体的验证错误项
//! - [`ValidationWarning`]：验证警告信息
//! - [`ValidationSeverity`]：验证问题的严重级别
//!
//! ## 错误分类
//!
//! ### 系统级错误
//! - I/O 错误：文件读写、权限问题
//! - 并发错误：锁竞争、资源冲突
//! - 资源错误：内存不足、超时
//!
//! ### 数据级错误
//! - 格式错误：不符合 TOML/JSON 规范
//! - 解析错误：无法解析为目标结构
//! - 类型错误：类型不匹配
//!
//! ### 业务级错误
//! - 验证错误：不符合业务规则
//! - 冲突错误：配置项相互冲突
//! - 逻辑错误：业务约束违反
//!
//! ## 使用示例
//!
//! ```rust
//! use lianpkg::core::liancfg::error::{ConfigError, ValidationResult};
//!
//! fn validate_config() -> Result<(), ConfigError> {
//!     // 验证逻辑
//!     Err(ConfigError::Validation("缺少必需字段 'name'".to_string()))
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// 配置错误枚举
#[derive(Debug, Error)]
pub enum ConfigError {
    /// 文件系统错误
    #[error("文件系统错误: {0}")]
    Io(#[from] std::io::Error),
    /// 配置格式错误
    #[error("配置格式错误: {0}")]
    Format(String),
    /// 解析错误
    #[error("解析错误: {0}")]
    Parse(String),
    /// 验证错误
    #[error("验证失败: {0}")]
    Validation(String),
    /// 类型不匹配
    #[error("类型不匹配: 期望 {expected}, 实际 {actual}")]
    TypeMismatch { expected: String, actual: String },
    /// 路径错误
    #[error("路径错误: {path} - {reason}")]
    PathError { path: PathBuf, reason: String },
    /// 不支持的操作
    #[error("不支持的操作: {0}")]
    Unsupported(String),
    /// 配置冲突
    #[error("配置冲突: {0}")]
    Conflict(String),
    /// 并发错误
    #[error("并发错误: {0}")]
    Concurrency(String),
    /// 超时错误
    #[error("操作超时")]
    Timeout,
    /// 资源不足
    #[error("资源不足: {0}")]
    Resource(String),
    /// 业务逻辑错误
    #[error("业务错误: {0}")]
    Business(String),
}

/// 验证结果
///
/// 包含配置验证的完整信息，包括：
/// - 整体验证状态（通过/失败）
/// - 所有发现的错误
/// - 所有警告信息
///
/// ## 使用说明
///
/// - `is_valid` 为 `true` 时，`errors` 应为空
/// - 即使 `is_valid` 为 `true`，仍可能有警告信息
/// - 建议在配置加载后立即验证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 验证是否通过
    pub is_valid: bool,
    /// 错误列表
    pub errors: Vec<ValidationError>,
    /// 警告列表
    pub warnings: Vec<ValidationWarning>,
}

/// 验证错误类型
///
/// 描述配置验证过程中发现的具体错误，包含：
/// - 错误位置（字段路径）
/// - 错误描述
/// - 严重级别
/// - 错误代码（用于程序化处理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// 错误字段路径（如 "server.port"）
    pub path: String,
    /// 错误描述信息
    pub message: String,
    /// 严重级别
    pub severity: ValidationSeverity,
    /// 错误代码（如 "E001"）
    pub code: String,
}

/// 验证警告
///
/// 表示潜在问题或不推荐的配置，但不会阻止配置加载。
/// 可选的 `suggestion` 字段提供改进建议。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// 警告字段路径
    pub path: String,
    /// 警告信息
    pub message: String,
    /// 改进建议
    pub suggestion: Option<String>,
}

/// 验证严重级别
///
/// 用于标识验证问题的严重程度：
/// - `Error`：必须修复的错误，会导致配置无效
/// - `Warning`：应该修复的问题，可能导致非预期行为
/// - `Info`：信息性提示，不影响功能
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// 错误级别 - 必须修复
    Error,
    /// 警告级别 - 建议修复
    Warning,
    /// 信息级别 - 仅供参考
    Info,
}
