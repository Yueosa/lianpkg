//! 磁盘空间检查接口

use std::path::{Path, PathBuf};

use crate::core::disk::structs::{CheckSpaceInput, CheckSpaceOutput};
use crate::core::error::{CoreError, CoreResult};

/// 查找存在的父目录
///
/// 从给定路径开始，向上查找直到找到存在的目录
pub fn find_existing_parent(path: &Path) -> Option<PathBuf> {
    let mut check_path = path.to_path_buf();
    while !check_path.exists() {
        if let Some(parent) = check_path.parent() {
            check_path = parent.to_path_buf();
        } else {
            return None;
        }
    }
    Some(check_path)
}

/// 检查磁盘可用空间
///
/// 如果指定路径不存在，会自动查找存在的父目录进行检查
pub fn check_space(input: CheckSpaceInput) -> CoreResult<CheckSpaceOutput> {
    // 查找存在的路径
    let check_path = find_existing_parent(&input.path).ok_or_else(|| CoreError::NotFound {
        message: "Cannot find existing directory to check disk space".to_string(),
        path: Some(input.path.display().to_string()),
    })?;

    // 获取可用空间
    let available = fs2::available_space(&check_path).map_err(|e| CoreError::Io {
        message: format!("Failed to get available space: {}", e),
        path: Some(check_path.display().to_string()),
    })?;

    // 获取总空间
    let total = fs2::total_space(&check_path).map_err(|e| CoreError::Io {
        message: format!("Failed to get total space: {}", e),
        path: Some(check_path.display().to_string()),
    })?;

    Ok(CheckSpaceOutput {
        available,
        total,
        check_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_existing_parent() {
        // 测试存在的路径
        let existing = find_existing_parent(Path::new("/tmp"));
        assert!(existing.is_some());

        // 测试不存在的路径应该返回父目录
        let non_existing = find_existing_parent(Path::new("/tmp/non_existing_dir_12345"));
        assert!(non_existing.is_some());
    }
}
