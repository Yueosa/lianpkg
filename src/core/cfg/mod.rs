//! cfg 模块 - 配置文件与状态文件的 CRUD 操作
//!
//! 本模块提供 9 个核心接口：
//! - config.toml: create_config_toml, read_config_toml, update_config_toml, delete_config_toml
//! - state.json: create_state_json, read_state_json, write_state_json, delete_state_json
//! - 清理: clear_lianpkg

mod structs;  // 结构体定义
mod utl;      // 工具函数与默认值
mod config;   // config.toml 操作
mod state;    // state.json 操作
mod clear;    // 目录清理操作

// ============================================================================
// 导出所有结构体
// ============================================================================

// Config.toml 相关结构体
pub use structs::CreateConfigInput;
pub use structs::CreateConfigOutput;
pub use structs::ReadConfigInput;
pub use structs::ReadConfigOutput;
pub use structs::UpdateConfigInput;
pub use structs::UpdateConfigOutput;
pub use structs::DeleteConfigInput;
pub use structs::DeleteConfigOutput;

// State.json 相关结构体
pub use structs::CreateStateInput;
pub use structs::CreateStateOutput;
pub use structs::ReadStateInput;
pub use structs::ReadStateOutput;
pub use structs::WriteStateInput;
pub use structs::WriteStateOutput;
pub use structs::DeleteStateInput;
pub use structs::DeleteStateOutput;

// Clear 相关结构体
pub use structs::ClearInput;
pub use structs::ClearOutput;
pub use structs::DeletedItem;
pub use structs::ItemType;

// ============================================================================
// 导出 9 个接口函数
// ============================================================================

// config.toml 操作接口
pub use config::create_config_toml;
pub use config::read_config_toml;
pub use config::update_config_toml;
pub use config::delete_config_toml;

// state.json 操作接口
pub use state::create_state_json;
pub use state::read_state_json;
pub use state::write_state_json;
pub use state::delete_state_json;

// 目录清理接口
pub use clear::clear_lianpkg;
