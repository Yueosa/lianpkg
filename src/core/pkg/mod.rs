//! pkg 模块 - Pkg 文件解析与解包
//!
//! 支持两种使用模式：
//! - 单独使用：parse_pkg 预览，unpack_pkg 一键解包
//! - 复合流程：parse_pkg → 判断 → unpack_entry 选择性解包
//!
//! 主要接口：
//! - 解析: parse_pkg
//! - 解包: unpack_pkg, unpack_entry

mod structs;
mod parse;
mod unpack;
mod utl;

// ============================================================================
// 导出 Input/Output 结构体
// ============================================================================

// 解析相关
pub use structs::ParsePkgInput;
pub use structs::ParsePkgOutput;

// 解包相关
pub use structs::UnpackPkgInput;
pub use structs::UnpackPkgOutput;
pub use structs::UnpackEntryInput;
pub use structs::UnpackEntryOutput;

// ============================================================================
// 导出运行时结构体
// ============================================================================
pub use structs::PkgInfo;
pub use structs::PkgEntry;
pub use structs::ExtractedFile;

// ============================================================================
// 导出解析接口
// ============================================================================
pub use parse::parse_pkg;

// ============================================================================
// 导出解包接口
// ============================================================================
pub use unpack::unpack_pkg;
pub use unpack::unpack_entry;
