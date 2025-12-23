use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub wallpaper: WallpaperConfig,
    pub unpack: UnpackConfig,
    pub tex: TexConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperConfig {
    // Steam Workshop 壁纸下载路径 (输入)
    pub workshop_path: String,
    // 不需要解包的壁纸输出路径 (输出 1)
    pub raw_output_path: String,
    // 需要解包的 .pkg 文件暂存路径 (中间文件)
    pub pkg_temp_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnpackConfig {
    // 解包后的文件输出路径 (输出 2)
    pub unpacked_output_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TexConfig {
    // .tex 转换后的图片输出路径 (输出 3)
    // 如果为 None，则默认在 unpacked_output_path 下的子目录
    pub converted_output_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let workshop_path = get_default_search_path();

        #[cfg(target_os = "windows")]
        let (raw_output_path, pkg_temp_path, unpacked_output_path) = (
            r".\Wallpapers_Raw".to_string(),
            r".\Pkg_Temp".to_string(),
            r".\Pkg_Unpacked".to_string(),
        );

        #[cfg(not(target_os = "windows"))]
        let (raw_output_path, pkg_temp_path, unpacked_output_path) = (
            "~/.local/share/lianpkg/Wallpapers_Raw".to_string(),
            "~/.local/share/lianpkg/Pkg_Temp".to_string(),
            "~/.local/share/lianpkg/Pkg_Unpacked".to_string(),
        );

        Config {
            wallpaper: WallpaperConfig {
                workshop_path,
                raw_output_path,
                pkg_temp_path,
            },
            unpack: UnpackConfig {
                unpacked_output_path,
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

pub fn load_config() -> ConfigStatus {
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
        
        // Add comments to the generated TOML
        let commented_config = format!(r#"# LianPkg Configuration File / LianPkg 配置文件

[wallpaper]
# Steam Workshop 壁纸下载路径 (输入)
# Windows 默认: C:\Program Files (x86)\Steam\steamapps\workshop\content\431960
# Linux 默认: ~/.local/share/Steam/steamapps/workshop/content/431960
workshop_path = "{}"

# 不需要解包的壁纸输出路径 (输出 1)
# Windows 默认: .\Wallpapers_Raw
# Linux 默认: ~/.local/share/lianpkg/Wallpapers_Raw
raw_output_path = "{}"

# 需要解包的 .pkg 文件暂存路径 (中间文件)
# Windows 默认: .\Pkg_Temp
# Linux 默认: ~/.local/share/lianpkg/Pkg_Temp
pkg_temp_path = "{}"

[unpack]
# 解包后的文件输出路径 (输出 2)
# Windows 默认: .\Pkg_Unpacked
# Linux 默认: ~/.local/share/lianpkg/Pkg_Unpacked
unpacked_output_path = "{}"

[tex]
# .tex 转换后的图片输出路径 (输出 3)
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
        match fs::read_to_string(&default_path) {
            Ok(content) => {
                match toml::from_str::<Config>(&content) {
                    Ok(c) => final_config = c,
                    Err(e) => eprintln!("[WARN] Failed to parse default.toml: {}", e),
                }
            }
            Err(e) => eprintln!("[WARN] Failed to read default.toml: {}", e),
        }
    }

    if user_exists {
        match fs::read_to_string(&user_path) {
            Ok(content) => {
                match toml::from_str::<toml::Value>(&content) {
                    Ok(user_val) => {
                        let mut config_val = match toml::to_string(&final_config) {
                            Ok(s) => match toml::from_str::<toml::Value>(&s) {
                                Ok(v) => v,
                                Err(e) => return ConfigStatus::Error(format!("Internal error parsing config: {}", e)),
                            },
                            Err(e) => return ConfigStatus::Error(format!("Internal error serializing config: {}", e)),
                        };

                        merge_toml_values(&mut config_val, user_val);

                        match config_val.try_into::<Config>() {
                            Ok(c) => final_config = c,
                            Err(e) => return ConfigStatus::Error(format!("Failed to merge config.toml: {}", e)),
                        }
                    }
                    Err(e) => return ConfigStatus::Error(format!("Failed to parse config.toml: {}", e)),
                }
            }
            Err(e) => return ConfigStatus::Error(format!("Failed to read config.toml: {}", e)),
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

#[cfg(target_os = "windows")]
fn get_default_search_path() -> String {
    use winreg::enums::*;
    use winreg::RegKey;

    // Try to find Steam path from Registry
    // 尝试从注册表查找 Steam 路径
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(steam) = hkcu.open_subkey("Software\\Valve\\Steam") {
        if let Ok(path) = steam.get_value::<String, _>("SteamPath") {
            let path = PathBuf::from(path);
            return path.join("steamapps")
                        .join("workshop")
                        .join("content")
                        .join("431960")
                        .to_string_lossy()
                        .to_string();
        }
    }
    
    // Fallback default
    r"C:\Program Files (x86)\Steam\steamapps\workshop\content\431960".to_string()
}

#[cfg(not(target_os = "windows"))]
fn get_default_search_path() -> String {
    "~/.local/share/Steam/steamapps/workshop/content/431960".to_string()
}
