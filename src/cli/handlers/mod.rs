//! handlers 模块 - 各命令处理器

pub mod wallpaper;
pub mod pkg;
pub mod tex;
pub mod auto;
pub mod config;
pub mod status;
pub mod show;

use super::output as out;
use lianpkg::api::native::context::{self, AppContext};
use std::path::PathBuf;

/// 公共初始化辅助：加载配置并返回 AppContext
///
/// 所有 handler 的 `run()` 开头统一调用此函数，
/// 消除 6 处重复的配置初始化样板。
pub fn init_context(config_path: Option<PathBuf>) -> Result<AppContext, String> {
    out::debug_api_enter(
        "native",
        "init",
        &format!("config_path={:?}", config_path),
    );
    let config_dir = config_path.map(|p| p.parent().unwrap_or(&p).to_path_buf());
    let ctx = context::init(config_dir).map_err(|e| e.to_string())?;
    out::debug_api_return(&format!("config_path={}", ctx.config_path.display()));
    Ok(ctx)
}
