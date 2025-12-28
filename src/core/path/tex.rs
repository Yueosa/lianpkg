//! Tex 路径处理

use std::path::{Path, PathBuf};
use crate::core::path::utl::expand_path;

/// 解析 Tex 转换输出目录
/// 
/// # 参数
/// - `converted_output_path`: 自定义输出路径（可选）
/// - `scene_root`: 场景根目录
/// - `input_file`: 输入文件路径（可选）
/// - `relative_base`: 相对路径基准目录（可选）
/// 
/// # 返回
/// 计算后的输出目录路径
pub fn resolve_tex_output_dir(
    converted_output_path: Option<&str>,
    scene_root: &Path,
    input_file: Option<&Path>,
    relative_base: Option<&Path>,
) -> PathBuf {
    // 确定基础输出目录：Pkg_Unpacked/壁纸ID/tex_converted/
    let base_dir = if let Some(custom_path) = converted_output_path {
        expand_path(custom_path)
            .join(scene_root.file_name().unwrap_or_default())
            .join("tex_converted")
    } else {
        scene_root.join("tex_converted")
    };

    // 如果有输入文件和相对基准，保持目录结构
    if let (Some(file), Some(base)) = (input_file, relative_base) {
        if let Ok(relative) = file.strip_prefix(base) {
            if let Some(parent) = relative.parent() {
                return base_dir.join(parent);
            }
        }
    }
    
    base_dir
}
