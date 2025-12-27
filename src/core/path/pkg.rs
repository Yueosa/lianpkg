//! Pkg 路径处理

/// 生成 Pkg 临时目标文件名
/// 格式: {目录名}_{文件名}
pub fn pkg_temp_dest(dir_name: &str, file_name: &str) -> String {
    format!("{}_{}", dir_name, file_name)
}

/// 从 Pkg 文件名中提取场景名
/// 例如: "12345_scene.pkg" -> "12345"
pub fn scene_name_from_pkg_stem(stem: &str) -> String {
    if let Some((prefix, _)) = stem.split_once('_') {
        prefix.to_string()
    } else {
        stem.to_string()
    }
}
