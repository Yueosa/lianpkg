//! 结构体定义 - 所有接口的入参与返回值结构体

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Config.toml 相关结构体
// ============================================================================

/// create_config_toml 接口入参
#[derive(Debug, Clone)]
pub struct CreateConfigInput {
    /// 配置文件路径
    pub path: PathBuf,
    /// 配置文件内容，None 则使用默认模板
    pub content: Option<String>,
}

/// create_config_toml 接口返回值
#[derive(Debug, Clone, Serialize)]
pub struct CreateConfigOutput {
    /// 是否触发了创建操作（文件已存在时为 false）
    pub created: bool,
    /// 配置文件路径
    pub path: PathBuf,
}

/// read_config_toml 接口入参
#[derive(Debug, Clone)]
pub struct ReadConfigInput {
    /// 配置文件路径
    pub path: PathBuf,
}

/// read_config_toml 接口返回值（成功时返回内容，失败通过 CoreError 返回）
#[derive(Debug, Clone, Serialize)]
pub struct ReadConfigOutput {
    /// 读取到的内容
    pub content: String,
}

/// update_config_toml 接口入参
#[derive(Debug, Clone)]
pub struct UpdateConfigInput {
    /// 配置文件路径
    pub path: PathBuf,
    /// 项名（支持点号分隔的嵌套键，如 "wallpaper.workshop_path"）
    pub key: String,
    /// 项值
    pub value: String,
}

/// update_config_toml 接口返回值（成功时返回内容，失败通过 CoreError 返回）
#[derive(Debug, Clone, Serialize)]
pub struct UpdateConfigOutput {
    /// 更新后的完整内容
    pub content: String,
}

/// delete_config_toml 接口入参
#[derive(Debug, Clone)]
pub struct DeleteConfigInput {
    /// 配置文件路径
    pub path: PathBuf,
}

/// delete_config_toml 接口返回值
#[derive(Debug, Clone, Serialize)]
pub struct DeleteConfigOutput {
    /// 是否触发了删除操作（文件不存在时为 false）
    pub deleted: bool,
    /// 删除的文件路径
    pub path: PathBuf,
}

// ============================================================================
// State.json 相关结构体
// ============================================================================

/// State.json 完整结构（用于序列化/反序列化）
///
/// `processed` 使用 HashMap<wallpaper_id, ProcessedEntry> 实现 O(1) 查重与去重。
/// 旧版 Vec 格式的 state.json 将无法解析，但 load_state 会 fallback 到空 StateData（可接受）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateData {
    /// 已处理壁纸映射: wallpaper_id → ProcessedEntry
    #[serde(default)]
    pub processed: HashMap<String, ProcessedEntry>,
    /// 上次运行时间（ISO 8601 字符串，如 "2025-01-15T08:30:00Z"）
    #[serde(default)]
    pub last_run: Option<String>,
}

/// 已处理的壁纸记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedEntry {
    /// 壁纸标题（从 project.json 读取）
    pub title: Option<String>,
    /// 处理类型
    pub process_type: ProcessType,
    /// 处理时间（ISO 8601 字符串）
    pub processed_at: String,
    /// 输出路径
    pub output_path: Option<String>,
}

/// 壁纸处理类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessType {
    /// 原始壁纸（直接复制）
    Raw,
    /// Pkg 壁纸（已解包）
    Pkg,
    /// Pkg + Tex 壁纸（已解包并转换）
    PkgTex,
    /// 跳过（不符合条件）
    Skipped,
}

/// create_state_json 接口入参
#[derive(Debug, Clone)]
pub struct CreateStateInput {
    /// 状态文件路径
    pub path: PathBuf,
    /// 状态文件内容，None 则使用默认模板 "{}"
    pub content: Option<String>,
}

/// create_state_json 接口返回值
#[derive(Debug, Clone, Serialize)]
pub struct CreateStateOutput {
    /// 是否触发了创建操作（文件已存在时为 false）
    pub created: bool,
    /// 状态文件路径
    pub path: PathBuf,
}

/// read_state_json 接口入参
#[derive(Debug, Clone)]
pub struct ReadStateInput {
    /// 状态文件路径
    pub path: PathBuf,
}

/// read_state_json 接口返回值（成功时返回内容，失败通过 CoreError 返回）
#[derive(Debug, Clone, Serialize)]
pub struct ReadStateOutput {
    /// 读取到的内容
    pub content: String,
}

/// write_state_json 接口入参
#[derive(Debug, Clone)]
pub struct WriteStateInput {
    /// 状态文件路径
    pub path: PathBuf,
    /// 覆写内容
    pub content: String,
}

/// write_state_json 接口返回值（成功时返回内容，失败通过 CoreError 返回）
#[derive(Debug, Clone, Serialize)]
pub struct WriteStateOutput {
    /// 覆写后的内容
    pub content: String,
}

/// delete_state_json 接口入参
#[derive(Debug, Clone)]
pub struct DeleteStateInput {
    /// 状态文件路径
    pub path: PathBuf,
}

/// delete_state_json 接口返回值
#[derive(Debug, Clone, Serialize)]
pub struct DeleteStateOutput {
    /// 是否触发了删除操作（文件不存在时为 false）
    pub deleted: bool,
    /// 删除的文件路径
    pub path: PathBuf,
}

// ============================================================================
// Clear 相关结构体
// ============================================================================

/// clear_lianpkg 接口入参
#[derive(Debug, Clone)]
pub struct ClearInput {
    /// 需要删除的目录路径
    pub dir_path: PathBuf,
}

/// clear_lianpkg 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearOutput {
    /// 是否触发了删除操作（目录不存在时为 false）
    pub cleared: bool,
    /// 删除的具体文件/目录列表
    pub deleted_items: Vec<DeletedItem>,
}

/// 删除项信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedItem {
    /// 删除的文件/目录路径
    pub path: PathBuf,
    /// 项类型
    pub item_type: ItemType,
}

/// 项类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemType {
    /// 文件
    File,
    /// 目录
    Directory,
}
