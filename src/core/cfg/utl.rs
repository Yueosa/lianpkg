//! 工具函数与默认值定义

use std::path::Path;
use std::fs;
use crate::core::path;

/// 生成 config.toml 的默认模板内容
/// 使用 core/path 模块获取平台相关的默认路径
pub fn default_config_template() -> String {
    let wp = path::default_workshop_path();
    let raw_out = path::default_raw_output_path();
    let pkg_temp = path::default_pkg_temp_path();
    let enable_raw = true;
    let unpack_out = path::default_unpacked_output_path();
    let clean_pkg_temp = true;
    let clean_unpacked = true;
    let converted_hint = String::new();

    format!(r#"# === LianPkg Configuration File / LianPkg 配置文件 ===

[wallpaper]
# === Steam Workshop 壁纸下载路径 ===
#     本程序将会从这个路径下扫描 wallpaper 壁纸
#         - Windows 默认: C:\\Program Files (x86)\\Steam\\steamapps\\workshop\\content\\431960
#         - Linux 默认: ~/.local/share/Steam/steamapps/workshop/content/431960
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

# === 需要解包的 .pkg 文件暂存路径 === 
#     为了不影响 wallpaper 结构, 本程序将会复制一份 .pkg 到这个临时文件夹
#     解包完成后就会清空, 如果你需要保留 .pkg 源文件可以在下面配置 clean_pkg_temp = false
#         - Windows 默认: .\\Pkg_Temp
#         - Linux 默认: ~/.local/share/lianpkg/Pkg_Temp
pkg_temp_path = "{pkg_temp}"


[unpack]
# === 解包后的文件输出路径 ===
#     这是 .pkg 文件第一次解包后的产物路径(不是最终产物), 如果需要你需要保留可以在下面配置 clean_unpacked = false
#         - Windows 默认: .\\Pkg_Unpacked
#         - Linux 默认: ~/.local/share/lianpkg/Pkg_Unpacked
unpacked_output_path = "{unpack_out}"

# === 是否在结束时清理 Pkg_Temp 目录===
clean_pkg_temp = {clean_pkg_temp}

# === 是否在结束时清理 Pkg_Unpacked 中除 tex_converted 以外的内容 ===
clean_unpacked = {clean_unpacked}


[tex]
# === .tex 转换后的图片输出路径 (输出 3) ===
#     这是最终产物的目录, 可以不配置, 也可以配置到指定路径
#     如果留空，则默认在解包路径下的 tex_converted 子目录中
# converted_output_path = "{converted_hint}"
"#)
}

/// 生成 state.json 的默认模板内容
pub fn default_state_template() -> String {
    "{}".to_string()
}

/// 确保目录存在，不存在则递归创建
pub fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|e| format!("Failed to create dir {}: {}", path.display(), e))
}
