//! 工具函数与默认值定义

use std::path::Path;

use crate::core::path;

/// 转义路径字符串用于 TOML（Windows 反斜杠需要转义）
fn escape_path_for_toml(path: &str) -> String {
    path.replace('\\', "\\\\")
}

/// 生成 config.toml 的默认模板内容
/// 使用 core/path 模块获取平台相关的默认路径
pub fn default_config_template() -> String {
    // workshop_path: 留空表示自动探测
    let wp = path::detect_workshop_path()
        .map(|p| escape_path_for_toml(&p.display().to_string()))
        .unwrap_or_default();
    let raw_out = escape_path_for_toml(&path::default_raw_output_path());
    let enable_raw = true;
    let unpack_out = escape_path_for_toml(&path::default_unpacked_output_path());
    let clean_unpacked = true;
    let converted_out = escape_path_for_toml(&path::default_converted_output_path());

    format!(
        r#"# === LianPkg Configuration File / LianPkg 配置文件 ===
#
# 路径格式说明 / Path Format:
#   - 在此配置文件中，Windows 路径的反斜杠需要转义: C:\\Users\\Name\\...
#   - 或者使用正斜杠（推荐）: C:/Users/Name/...
#   - 命令行参数 (--search, --output 等) 可直接使用标准格式: C:\Users\Name\...
#
# Path format notes:
#   - In this config file, Windows backslashes must be escaped: C:\\Users\\Name\\...
#   - Or use forward slashes (recommended): C:/Users/Name/...
#   - CLI arguments (--search, --output, etc.) accept standard format: C:\Users\Name\...

[wallpaper]
# === Steam Workshop 壁纸下载路径 ===
#     本程序将会从这个路径下扫描 wallpaper 壁纸
#         - Windows 默认: C:\\Program Files (x86)\\Steam\\steamapps\\workshop\\content\\431960
#         - Linux 默认: ~/.local/share/Steam/steamapps/workshop/content/431960
# 留空则自动探测 Steam Workshop 路径
workshop_path = "{wp}"

# === 不需要解包的壁纸输出路径 ===
#     有些 wallpaper 壁纸不需要解包, 就会放到这个路径下
#         - Windows 默认: .\\Wallpapers_Raw
#         - Linux 默认: ~/.local/share/lianpkg/Wallpapers_Raw
raw_output_path = "{raw_out}"

# === 是否提取原始壁纸（非 pkg 文件） ===
#     如果设置为 false，将跳过复制非 pkg 壁纸到 raw_output_path
#     Default/默认: true
enable_raw_output = {enable_raw}


[unpack]
# === PKG 解包的基础路径 ===
#     PKG 解包产物放在: <此路径>/<壁纸ID>/
#         - Windows 默认: .\\Pkg_Unpacked
#         - Linux 默认: ~/.local/share/lianpkg/Pkg_Unpacked
unpacked_output_path = "{unpack_out}"

# === 是否在结束时清理解包中间文件 ===
#     启用后，流水线完成后会自动删除 unpacked_output_path 下的所有内容
#     Default/默认: true
clean_unpacked = {clean_unpacked}


[tex]
# === TEX 转换后的图片/视频输出路径 ===
#     转换产物放在: <此路径>/<壁纸ID>/
#         - Windows 默认: .\\Pkg_Unpacked\\tex_converted
#         - Linux 默认: ~/.local/share/lianpkg/Pkg_Unpacked/tex_converted
converted_output_path = "{converted_out}"


[pipeline]
# === 是否启用增量处理 ===
#     启用后，已处理过的壁纸将被跳过（根据 state.json 记录判断）
#     Default/默认: false
incremental = false

# === 是否在流水线中自动执行 pkg 解包 ===
#     Default/默认: true
auto_unpack_pkg = true

# === 是否在流水线中自动执行 tex 转换 ===
#     Default/默认: true
auto_convert_tex = true
"#
    )
}

/// 生成 state.json 的默认模板内容
pub fn default_state_template() -> String {
    r#"{"processed":{}}"#.to_string()
}

/// 确保目录存在，不存在则递归创建
pub fn ensure_dir(path: &Path) -> crate::core::error::CoreResult<()> {
    std::fs::create_dir_all(path).map_err(|e| {
        crate::core::error::CoreError::io_with_path(e.to_string(), path.display().to_string())
    })
}
