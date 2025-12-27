//! 结构体定义 - 所有接口的入参与返回值结构体

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone)]
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

/// read_config_toml 接口返回值
#[derive(Debug, Clone)]
pub struct ReadConfigOutput {
    /// 是否读取成功
    pub success: bool,
    /// 读取到的内容，失败时为 None
    pub content: Option<String>,
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

/// update_config_toml 接口返回值
#[derive(Debug, Clone)]
pub struct UpdateConfigOutput {
    /// 更新是否成功
    pub success: bool,
    /// 更新后的完整内容，失败时为 None
    pub content: Option<String>,
}

/// delete_config_toml 接口入参
#[derive(Debug, Clone)]
pub struct DeleteConfigInput {
    /// 配置文件路径
    pub path: PathBuf,
}

/// delete_config_toml 接口返回值
#[derive(Debug, Clone)]
pub struct DeleteConfigOutput {
    /// 是否触发了删除操作（文件不存在时为 false）
    pub deleted: bool,
    /// 删除的文件路径
    pub path: PathBuf,
}

// ============================================================================
// State.json 相关结构体
// ============================================================================

/// create_state_json 接口入参
#[derive(Debug, Clone)]
pub struct CreateStateInput {
    /// 状态文件路径
    pub path: PathBuf,
    /// 状态文件内容，None 则使用默认模板 "{}"
    pub content: Option<String>,
}

/// create_state_json 接口返回值
#[derive(Debug, Clone)]
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

/// read_state_json 接口返回值
#[derive(Debug, Clone)]
pub struct ReadStateOutput {
    /// 是否读取成功
    pub success: bool,
    /// 读取到的内容，失败时为 None
    pub content: Option<String>,
}

/// write_state_json 接口入参
#[derive(Debug, Clone)]
pub struct WriteStateInput {
    /// 状态文件路径
    pub path: PathBuf,
    /// 覆写内容
    pub content: String,
}

/// write_state_json 接口返回值
#[derive(Debug, Clone)]
pub struct WriteStateOutput {
    /// 覆写是否成功
    pub success: bool,
    /// 覆写后的内容，失败时为 None
    pub content: Option<String>,
}

/// delete_state_json 接口入参
#[derive(Debug, Clone)]
pub struct DeleteStateInput {
    /// 状态文件路径
    pub path: PathBuf,
}

/// delete_state_json 接口返回值
#[derive(Debug, Clone)]
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
