//! core 模块 - 核心业务逻辑
//!
//! 本模块包含 LianPkg 的所有核心功能实现：
//! - error: 统一错误处理
//! - path: 路径处理与解析
//! - disk: 磁盘空间检查
//! - cfg: 配置文件与状态文件 CRUD 操作
//! - paper: Wallpaper 壁纸扫描与复制
//! - pkg: Pkg 文件解析与解包
//! - tex: Tex 文件解析与转换

pub mod cfg; // 配置文件与状态文件操作
pub mod disk; // 磁盘空间检查
pub mod error; // 统一错误处理（其他模块依赖此模块，必须首先声明）
pub mod paper; // Wallpaper 壁纸扫描与复制
pub mod path; // 路径处理与解析
pub mod pkg; // Pkg 文件解析与解包
pub mod tex; // Tex 文件解析与转换
