use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub wallpaper: WallpaperConfig,
    pub unpack: UnpackConfig,
    pub tex: TexConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperConfig {
    pub workshop_path: String,
    pub raw_output_path: String,
    pub pkg_temp_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnpackConfig {
    pub unpacked_output_path: String,
    pub clean_pkg_temp: bool,
    pub clean_unpacked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TexConfig {
    pub converted_output_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let workshop_path = path::default_workshop_path();
        let raw_output_path = path::default_raw_output_path();
        let pkg_temp_path = path::default_pkg_temp_path();
        let unpacked_output_path = path::default_unpacked_output_path();

        Config {
            wallpaper: WallpaperConfig {
                workshop_path,
                raw_output_path,
                pkg_temp_path,
            },
            unpack: UnpackConfig {
                unpacked_output_path,
                clean_pkg_temp: true,
                clean_unpacked: true,
            },
            tex: TexConfig {
                converted_output_path: None,
            },
        }
    }
}

pub enum ConfigStatus {
    Loaded(Config),
    CreatedDefault(PathBuf),
    Error(String),
}

pub fn load_config(custom_path: Option<PathBuf>) -> ConfigStatus {
    if let Some(path) = custom_path {
        let mut final_config = Config::default();
        if !path.exists() {
            return ConfigStatus::Error(format!("Config file not found at {}", path.display()));
        }

        if let Err(e) = merge_config_file(&path, &mut final_config) {
            return ConfigStatus::Error(e);
        }
        return ConfigStatus::Loaded(final_config);
    }

    let config_dir = if cfg!(target_os = "windows") {
        match std::env::current_exe() {
            Ok(path) => path.parent().unwrap_or(&path).to_path_buf(),
            Err(e) => return ConfigStatus::Error(format!("Failed to get current exe path: {}", e)),
        }
    } else {
        match dirs::config_dir() {
            Some(d) => d.join("lianpkg"),
            None => return ConfigStatus::Error("Could not determine config directory".to_string()),
        }
    };

    if !config_dir.exists() {
        if let Err(e) = fs::create_dir_all(&config_dir) {
            return ConfigStatus::Error(format!("Failed to create config dir: {}", e));
        }
    }

    let default_path = config_dir.join("default.toml");
    let user_path = config_dir.join("config.toml");

    let default_exists = default_path.exists();
    let user_exists = user_path.exists();

    if !default_exists && !user_exists {
        let default_config = Config::default();
        
        let commented_config = format!(r#"# LianPkg Configuration File / LianPkg 配置文件

[wallpaper]
# Steam Workshop 壁纸下载路径
# 本程序将会从这个路径下扫描 wallpaper 壁纸
# Windows 默认: C:\Program Files (x86)\Steam\steamapps\workshop\content\431960
# Linux 默认: ~/.local/share/Steam/steamapps/workshop/content/431960

workshop_path = "{}"

# 不需要解包的壁纸输出路径
# 有些 wallpaper 壁纸不需要解包, 就会放到这个路径下
# Windows 默认: .\Wallpapers_Raw
# Linux 默认: ~/.local/share/lianpkg/Wallpapers_Raw

raw_output_path = "{}"

# 需要解包的 .pkg 文件暂存路径
# 为了不影响 wallpaper 结构, 本程序将会复制一份 .pkg 到这个临时文件夹
# 解包完成后就会清空, 如果你需要保留 .pkg 源文件可以在下面配置 clean_pkg_temp = false
# Windows 默认: .\Pkg_Temp
# Linux 默认: ~/.local/share/lianpkg/Pkg_Temp

pkg_temp_path = "{}"

[unpack]
# 解包后的文件输出路径
# 这是 .pkg 文件第一次解包后的产物路径(不是最终产物), 如果需要你需要保留可以在下面配置 clean_unpacked = false
# Windows 默认: .\Pkg_Unpacked
# Linux 默认: ~/.local/share/lianpkg/Pkg_Unpacked

unpacked_output_path = "{}"

# 是否在结束时清理 Pkg_Temp 目录
clean_pkg_temp = true

# 是否在结束时清理 Pkg_Unpacked 中除 tex_converted 以外的内容
clean_unpacked = true

[tex]
# .tex 转换后的图片输出路径 (输出 3)
# 这是最终产物的目录, 可以不配置, 也可以配置到指定路径
# 如果留空，则默认在解包路径下的 tex_converted 子目录中
# converted_output_path = "..."
"#, 
            default_config.wallpaper.workshop_path.replace("\\", "\\\\"),
            default_config.wallpaper.raw_output_path.replace("\\", "\\\\"),
            default_config.wallpaper.pkg_temp_path.replace("\\", "\\\\"),
            default_config.unpack.unpacked_output_path.replace("\\", "\\\\")
        );

        if let Err(e) = fs::write(&user_path, commented_config) {
            return ConfigStatus::Error(format!("Failed to write default config: {}", e));
        }
        return ConfigStatus::CreatedDefault(user_path);
    }

    let mut final_config = Config::default();

    if default_exists {
        if let Err(e) = merge_config_file(&default_path, &mut final_config) {
            eprintln!("[WARN] {}", e);
        }
    }

    if user_exists {
        if let Err(e) = merge_config_file(&user_path, &mut final_config) {
            return ConfigStatus::Error(e);
        }
    }

    ConfigStatus::Loaded(final_config)
}

fn merge_toml_values(base: &mut toml::Value, override_val: toml::Value) {
    match (base, override_val) {
        (toml::Value::Table(base_map), toml::Value::Table(override_map)) => {
            for (k, v) in override_map {
                if let Some(base_v) = base_map.get_mut(&k) {
                    merge_toml_values(base_v, v);
                } else {
                    base_map.insert(k, v);
                }
            }
        }
        (base, override_val) => {
            *base = override_val;
        }
    }
}

fn merge_config_file(path: &Path, final_config: &mut Config) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let user_val = toml::from_str::<toml::Value>(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

    let mut config_val = toml::to_string(final_config)
        .map_err(|e| format!("Internal error serializing config: {}", e))
        .and_then(|s| toml::from_str::<toml::Value>(&s)
            .map_err(|e| format!("Internal error parsing config: {}", e)))?;

    merge_toml_values(&mut config_val, user_val);

    *final_config = config_val
        .try_into::<Config>()
        .map_err(|e| format!("Failed to merge {}: {}", path.display(), e))?;

    Ok(())
}

