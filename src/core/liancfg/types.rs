//! 通用配置类型定义模块
//!
//! 本模块定义了配置系统中使用的核心数据结构与枚举类型，
//! 用于描述配置文件的格式、内容、元数据以及操作结果。
//!
//! ## 支持的配置格式
//!
//! - TOML (`.toml`)
//! - JSON (`.json`)
//!
//! 本模块本身 **不负责** 配置文件的读取或解析，
//! 仅提供与配置相关的 **类型抽象与语义约定**。
//!
//! ## 主要类型
//!
//! - [`ConfigType`]：配置文件格式标识
//! - [`ConfigValue`]：通用配置值枚举（类似 JSON Value）
//! - [`ConfigMap`]：结构化配置映射
//! - [`ConfigContent`]：配置原始内容或结构化内容
//! - [`ConfigMetadata`]：配置文件元信息
//! - [`ConfigResult`]：配置操作结果封装

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// 配置文件类型枚举 - 用于标识和分派
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigType {
    #[serde(rename = "toml")]
    /// TOML 文件类型
    Toml,
    #[serde(rename = "json")]
    /// JSON 文件类型
    Json,
}

/// 配置值类型 - 支持基本数据类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigValue {
    /// 字符串 - String
    String(String),
    /// 整数 - int64
    Integer(i64),
    /// 浮点数 - float64
    Float(f64),
    /// 布尔值 - bool
    Boolean(bool),
    /// 数组 - Vec<ConfigValue>
    Array(Vec<ConfigValue>),
    /// 对象 - ConfigMap
    Object(ConfigMap),
    /// 空值
    Null,
}

/// 配置映射表 - 用于结构化数据
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ConfigMap(pub std::collections::HashMap<String, ConfigValue>);

/// 配置内容容器 - 区分不同格式的原始内容
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigContent {
    /// TOML 格式的原始字符串
    /// 约束: 必须是有效的 TOML 语法
    Toml(String),
    /// JSON 格式的原始字符串
    /// 约束: 必须是有效的 JSON 语法
    Json(String),
    /// 结构化数据 (独立于格式)
    /// 约束: 必须能正确序列化为目标格式
    Structured(ConfigMap),
}

/// 配置元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// 文件路径
    /// 约束: 必须是有效的文件系统路径
    pub path: PathBuf,
    /// 配置类型
    pub config_type: ConfigType,
    /// 文件大小 (字节)
    pub size: u64,
    /// 最后修改时间
    pub last_modified: SystemTime,
    /// 文件哈希 (用于一致性检查)
    /// 约束: 可选, 如果提供必须是有效的 SHA256 哈希
    pub hash: Option<String>,
    /// 配置版本 (语义化版本)
    /// 约束: 如果提供必须符合 semver 格式
    pub version: Option<String>,
}

/// 操作结果封装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResult<T> {
    /// 操作是否成功
    pub success: bool,
    /// 成功时返回的数据
    pub data: Option<T>,
    /// 错误信息
    pub error: Option<String>,
    /// 操作耗时 (毫秒)
    pub duration_ms: u64,
    /// 时间戳
    pub timestamp: SystemTime,
}
