use std::fs;
use std::path::{Path, PathBuf};
use crate::core::config::structs::{Config, ConfigRaw, WallpaperConfigRaw, UnpackConfigRaw, TexConfigRaw};
use crate::core::path;

pub fn resolve_config_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|d| d.join("lianpkg"))
        .ok_or_else(|| "Could not determine config directory".to_string())
}

pub fn config_file_path(custom_path: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(p) = custom_path {
        return Ok(p);
    }
    Ok(resolve_config_dir()?.join("config.toml"))
}

pub fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|e| format!("Failed to create config dir {}: {}", path.display(), e))
}

pub fn default_raw_with_defaults() -> ConfigRaw {
    ConfigRaw {
        wallpaper: WallpaperConfigRaw {
            workshop_path: Some(path::default_workshop_path()),
            raw_output_path: Some(path::default_raw_output_path()),
            pkg_temp_path: Some(path::default_pkg_temp_path()),
            enable_raw_output: Some(true),
        },
        unpack: UnpackConfigRaw {
            unpacked_output_path: Some(path::default_unpacked_output_path()),
            clean_pkg_temp: Some(true),
            clean_unpacked: Some(true),
        },
        tex: TexConfigRaw {
            converted_output_path: None,
        },
    }
}

pub fn resolve_config(raw: &ConfigRaw) -> Config {
    let defaults = Config::default();

    let wallpaper = &raw.wallpaper;
    let unpack = &raw.unpack;
    let tex = &raw.tex;

    Config {
        wallpaper: crate::core::config::structs::WallpaperConfig {
            workshop_path: wallpaper
                .workshop_path
                .clone()
                .unwrap_or_else(|| defaults.wallpaper.workshop_path.clone()),
            raw_output_path: wallpaper
                .raw_output_path
                .clone()
                .unwrap_or_else(|| defaults.wallpaper.raw_output_path.clone()),
            pkg_temp_path: wallpaper
                .pkg_temp_path
                .clone()
                .unwrap_or_else(|| defaults.wallpaper.pkg_temp_path.clone()),
            enable_raw_output: wallpaper
                .enable_raw_output
                .unwrap_or(defaults.wallpaper.enable_raw_output),
        },
        unpack: crate::core::config::structs::UnpackConfig {
            unpacked_output_path: unpack
                .unpacked_output_path
                .clone()
                .unwrap_or_else(|| defaults.unpack.unpacked_output_path.clone()),
            clean_pkg_temp: unpack
                .clean_pkg_temp
                .unwrap_or(defaults.unpack.clean_pkg_temp),
            clean_unpacked: unpack
                .clean_unpacked
                .unwrap_or(defaults.unpack.clean_unpacked),
        },
        tex: crate::core::config::structs::TexConfig {
            converted_output_path: tex
                .converted_output_path
                .clone()
                .or_else(|| defaults.tex.converted_output_path.clone()),
        },
    }
}

pub fn merge_raw(base: &mut ConfigRaw, patch: &ConfigRaw) {
    if let Some(v) = &patch.wallpaper.workshop_path { base.wallpaper.workshop_path = Some(v.clone()); }
    if let Some(v) = &patch.wallpaper.raw_output_path { base.wallpaper.raw_output_path = Some(v.clone()); }
    if let Some(v) = &patch.wallpaper.pkg_temp_path { base.wallpaper.pkg_temp_path = Some(v.clone()); }
    if let Some(v) = patch.wallpaper.enable_raw_output { base.wallpaper.enable_raw_output = Some(v); }

    if let Some(v) = &patch.unpack.unpacked_output_path { base.unpack.unpacked_output_path = Some(v.clone()); }
    if let Some(v) = patch.unpack.clean_pkg_temp { base.unpack.clean_pkg_temp = Some(v); }
    if let Some(v) = patch.unpack.clean_unpacked { base.unpack.clean_unpacked = Some(v); }

    if let Some(v) = &patch.tex.converted_output_path { base.tex.converted_output_path = Some(v.clone()); }
}

pub fn load_raw(path: &Path) -> Result<ConfigRaw, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    toml::from_str::<ConfigRaw>(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn write_raw(path: &Path, raw: &ConfigRaw) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    let content = toml::to_string_pretty(raw)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(path, content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

/// Build a commented default config template using provided raw defaults.
pub fn build_default_config_template(raw: &ConfigRaw) -> String {
    let wp = raw.wallpaper.workshop_path.clone().unwrap_or_default();
    let raw_out = raw.wallpaper.raw_output_path.clone().unwrap_or_default();
    let pkg_temp = raw.wallpaper.pkg_temp_path.clone().unwrap_or_default();
    let enable_raw = raw.wallpaper.enable_raw_output.unwrap_or(true);

    let unpack_out = raw.unpack.unpacked_output_path.clone().unwrap_or_default();
    let clean_pkg_temp = raw.unpack.clean_pkg_temp.unwrap_or(true);
    let clean_unpacked = raw.unpack.clean_unpacked.unwrap_or(true);

    let converted_hint = raw.tex.converted_output_path.clone().unwrap_or_default();

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
