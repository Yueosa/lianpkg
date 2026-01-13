//! 配置命令定义模块
//!
//! 本模块定义了配置系统的**命令接口层**，
//! 用于将用户请求转换为可执行的配置操作。
//!
//! ## 核心类型
//!
//! - [`ConfigCommand`]：顶层命令枚举，包含所有支持的配置操作
//! - [`ListOperation`]：列出配置文件的参数
//! - [`SearchOperation`]：搜索配置内容的参数
//! - [`DiffOperation`]：比较配置差异的参数
//!
//! ## 设计说明
//!
//! `ConfigCommand` 是对 `operations` 模块中基础操作的扩展，
//! 增加了列表、搜索、比较等**高级查询操作**。
//!
//! 与 `operations::ConfigOperation` 的区别：
//! - `ConfigCommand`：面向用户的完整命令集（包括查询、管理类操作）
//! - `ConfigOperation`：面向批处理的基础操作集（仅包括 CRUD 操作）

use super::operations::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 配置命令枚举 - 用于操作分派
#[derive(Debug, Clone)]
pub enum ConfigCommand {
    /// 读取配置
    Read(ReadOperation),
    /// 写入配置
    Write(WriteOperation),
    /// 更新配置
    Update(UpdateOperation),
    /// 删除配置
    Delete(DeleteOperation),
    /// 验证配置
    Validate(ValidateOperation),
    /// 转换配置格式
    Convert(ConvertOperation),
    /// 批量操作
    Batch(BatchOperation),
    /// 列出所有配置
    List(ListOperation),
    /// 搜索配置
    Search(SearchOperation),
    /// 比较配置差异
    Diff(DiffOperation),
}

/// 列出配置的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOperation {
    /// 目录路径
    pub directory: PathBuf,
    /// 过滤模式 (glob)
    pub pattern: Option<String>,
    /// 是否递归查找
    pub recursive: bool,
    /// 是否包含文件元数据
    pub include_metadata: bool,
    /// 排序方式
    pub sort_by: ListSortBy,
}

/// 列表排序方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListSortBy {
    /// 名称
    Name,
    /// 大小
    Size,
    /// 修改时间
    Modified,
    /// 类型
    Type,
}

/// 搜索配置的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOperation {
    /// 搜索目录
    pub directory: PathBuf,
    /// 搜索关键词
    pub query: String,
    /// 是否搜索键名
    pub search_keys: bool,
    /// 是否搜索键值
    pub search_values: bool,
    /// 是否使用正则表达式
    pub use_regex: bool,
    /// 是否大小写敏感
    pub case_sensitive: bool,
    /// 最大结果数
    /// 约束: > 0
    pub max_results: usize,
}

/// 比较配置差异的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffOperation {
    /// 第一个文件路径
    pub left_path: PathBuf,
    /// 第二个文件路径
    pub right_path: PathBuf,
    /// 忽略的字段路径
    pub ignore_paths: Vec<String>,
    /// 比较深度
    /// 约束: >= 1
    pub max_depth: usize,
    /// 是否显示详细差异
    pub verbose: bool,
    /// 输出格式
    pub output_format: DiffOutputFormat,
}

/// 差异输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffOutputFormat {
    Text,
    Json,
    Yaml,
    Html,
}

/// 配置操作别名 - 用于简化常用操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigAction {
    Create,
    Read,
    Update,
    Delete,
    Validate,
    Convert,
}
