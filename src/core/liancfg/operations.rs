//! 配置操作参数定义模块
//!
//! 本模块定义了配置系统支持的各类**操作参数结构**，
//! 用于描述对配置文件执行的读、写、更新、删除、验证、转换
//! 以及批量处理等操作。
//!
//! 每一种 `*Operation` 结构体都表示一次**明确、可序列化的配置操作请求**，
//! 通常用于：
//!
//! - 命令行接口 (CLI) 参数解析
//! - API / RPC 请求体
//! - 内部任务调度与批处理系统
//!
//! ## 设计原则
//!
//! - 本模块仅描述“**做什么**”，不关心“**如何做**”
//! - 不包含任何 I/O 或业务逻辑
//! - 所有结构均可序列化 / 反序列化，便于跨边界传输
//!
//! ## 主要操作类型
//!
//! - [`ReadOperation`]：读取配置
//! - [`WriteOperation`]：写入配置
//! - [`UpdateOperation`]：更新配置
//! - [`DeleteOperation`]：删除配置
//! - [`ValidateOperation`]：验证配置
//! - [`ConvertOperation`]：格式转换
//! - [`BatchOperation`]：批量操作

use super::types::*;
use std::path::PathBuf;

/// 读取配置的参数
/// 约束: path 必须存在且可读, cache_ttl 必须 >= 0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadOperation {
    /// 配置文件路径
    pub path: PathBuf,
    /// 是否缓存结果
    pub use_cache: bool,
    /// 缓存生存时间 (秒)
    /// 约束: >= 0
    pub cache_ttl: u64,
    /// 是否验证配置完整性
    pub validate: bool,
}

/// 写入配置的参数
/// 约束: path 必须是有效路径, overwrite 为 false 时不能覆盖已有文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOperation {
    /// 配置文件路径
    pub path: PathBuf,
    /// 配置内容
    pub content: ConfigContent,
    /// 是否覆盖已存在文件
    pub overwrite: bool,
    /// 是否创建备份
    pub backup: bool,
    /// 备份文件后缀
    /// 约束: 不能包含路径分隔符
    pub backup_suffix: Option<String>,
    /// 是否格式化输出
    pub pretty_format: bool,
}

/// 更新配置文件的参数
/// 约束: updates 不能为空, merge_strategy 必须是有效值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOperation {
    /// 配置文件路径
    pub path: PathBuf,
    /// 更新内容
    pub updates: ConfigMap,
    /// 合并策略
    pub merge_strategy: MergeStrategy,
    /// 是否创建备份
    pub create_backup: bool,
    /// 是否深度合并 (对于嵌套对象)
    pub deep_merge: bool,
}

/// 合并策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// 替换整个配置
    Replace,
    /// 合并更新 (保留未提及字段)
    Merge,
    /// 仅更新已有字段
    UpdateOnly,
}

/// 删除配置的参数
/// 约束: path 必须存在且可写
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteOperation {
    /// 配置文件路径
    pub path: PathBuf,
    /// 是否移动到回收站 (非永久删除)
    pub move_to_trash: bool,
    /// 回收站路径 (如果 move_to_trash 为 true)
    pub trash_path: Option<PathBuf>,
    /// 确认标记 (防止误删)
    pub confirmation_token: Option<String>,
}

/// 验证配置的参数
/// 约束: schema_path 必须是有效路径或 None
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateOperation {
    /// 配置文件路径
    pub path: PathBuf,
    /// 验证模式路径 (Json Schema等)
    pub schema_path: Option<PathBuf>,
    /// 严格模式 (禁止未知字段)
    pub strict: bool,
    /// 是否验证类型一致性
    pub validate_types: bool,
    /// 允许的配置版本范围
    pub version_range: Option<String>,
}

/// 转换配置的参数
/// 约束: source_path 必须存在, target_type 不能与源类型相同
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOperation {
    /// 源文件路径
    pub source_path: PathBuf,
    /// 目标文件路径
    pub target_path: PathBuf,
    /// 目标配置类型
    pub target_type: ConfigType,
    /// 是否保留注释
    pub preserve_comments: bool,
    /// 是否美化输出
    pub pretty: bool,
    /// 转换失败时的行为
    pub on_error: ConvertErrorBehavior,
}

/// 转换错误处理策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvertErrorBehavior {
    /// 失败时中止
    Abort,
    /// 跳过错误字段
    Skip,
    /// 使用默认值替换
    UseDefault,
}

/// 配置操作枚举 - 用于批量操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigOperation {
    /// 读取操作
    Read(ReadOperation),
    /// 写入操作
    Write(WriteOperation),
    /// 更新操作
    Update(UpdateOperation),
    /// 删除操作
    Delete(DeleteOperation),
    /// 验证操作
    Validate(ValidateOperation),
    /// 转换操作
    Convert(ConvertOperation),
}

/// 批量操作参数
/// 约束: operations 不能为空
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    /// 操作列表
    pub operations: Vec<ConfigOperation>,
    /// 是否在遇到错误时停止
    pub stop_on_error: bool,
    /// 并行执行数量 (0 表示串行)
    /// 约束: >= 0
    pub parallelism: usize,
    /// 事务模式 (要么全成功, 要么全回滚)
    pub transactional: bool,
    /// 超时时间 (秒)
    /// 约束: > 0
    pub timeout_secs: u64,
}
