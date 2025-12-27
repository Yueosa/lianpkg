//! tex 模块 - Tex 文件解析与转换
//!
//! 支持两种使用模式：
//! - 单独使用：parse_tex 预览，convert_tex 一键转换
//! - 复合流程：parse_tex → 判断格式 → convert_tex
//!
//! 支持的格式：
//! - 压缩格式: DXT1, DXT3, DXT5
//! - 原始格式: RGBA8888, RG88, R8
//! - 图片格式: PNG, JPEG, BMP, GIF 等
//! - 视频格式: MP4

mod structs;
mod parse;
mod convert;
mod reader;
mod decoder;

// ============================================================================
// 导出 Input/Output 结构体
// ============================================================================
pub use structs::ParseTexInput;
pub use structs::ParseTexOutput;
pub use structs::ConvertTexInput;
pub use structs::ConvertTexOutput;

// ============================================================================
// 导出运行时结构体
// ============================================================================
pub use structs::TexInfo;
pub use structs::ConvertedFile;
pub use structs::MipmapFormat;

// ============================================================================
// 导出解析接口
// ============================================================================
pub use parse::parse_tex;

// ============================================================================
// 导出转换接口
// ============================================================================
pub use convert::convert_tex;
