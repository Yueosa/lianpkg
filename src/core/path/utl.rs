//! 通用路径工具函数

use std::fs;
use std::path::{Path, PathBuf};

/// 确保目录存在，不存在则递归创建
pub fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|e| format!("Failed to create dir {}: {}", path.display(), e))
}

/// 展开路径中的 `~` 为用户主目录
pub fn expand_path(path_str: &str) -> PathBuf {
    if path_str.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            if path_str == "~" {
                return home;
            }
            if path_str.starts_with("~/") {
                return home.join(&path_str[2..]);
            }
        }
    }
    PathBuf::from(path_str)
}

/// 获取唯一的输出路径
/// 如果目标路径已存在，会在文件名后添加 `-1`, `-2` 等后缀
pub fn get_unique_output_path(base: &Path, name: &str) -> PathBuf {
    let mut target = base.join(name);
    if !target.exists() {
        return target;
    }

    let mut i = 1;
    loop {
        let new_name = format!("{}-{}", name, i);
        target = base.join(&new_name);
        if !target.exists() {
            return target;
        }
        i += 1;
    }
}
