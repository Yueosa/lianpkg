//! 通用路径工具函数

use super::types::*;
use crate::core::error::{CoreError, CoreResult};
use std::fs;

/// 确保目录存在，不存在则递归创建
pub fn ensure_dir(input: EnsureDirInput) -> CoreResult<EnsureDirOutput> {
    let created = if input.path.exists() {
        false
    } else {
        fs::create_dir_all(&input.path).map_err(|e| {
            CoreError::io_with_path(e.to_string(), input.path.display().to_string())
        })?;
        true
    };

    Ok(EnsureDirOutput {
        path: input.path,
        created,
    })
}

/// 展开路径中的 `~` 为用户主目录
pub fn expand_path(input: ExpandPathInput) -> CoreResult<ExpandPathOutput> {
    let path = if input.path.starts_with("~") {
        let home =
            dirs::home_dir().ok_or_else(|| CoreError::not_found("Home directory not found"))?;

        if input.path == "~" {
            home
        } else if input.path.starts_with("~/") {
            home.join(&input.path[2..])
        } else {
            std::path::PathBuf::from(&input.path)
        }
    } else {
        std::path::PathBuf::from(&input.path)
    };

    Ok(ExpandPathOutput { path })
}
