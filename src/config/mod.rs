use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub wallpaper: WallpaperConfig,
    pub pkg: PkgConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperConfig {
    pub search_path: String,
    pub output_path: String,
    pub video_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PkgConfig {
    pub output_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            wallpaper: WallpaperConfig {
                search_path: "~/.local/share/Steam/steamapps/workshop/content/431960".to_string(),
                output_path: "~/.local/share/lianpkg/".to_string(),
                video_path: None,
            },
            pkg: PkgConfig {
                output_path: "~/.local/share/lianpkg/Pkg_converted".to_string(),
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
    let config_dir = match dirs::config_dir() {
        Some(d) => d.join("lianpkg"),
        None => return ConfigStatus::Error("Could not determine config directory".to_string()),
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
        let default_config_content = r#"# LianPkg Configuration File / LianPkg 配置文件

[wallpaper]
# The path where Wallpaper Engine downloads wallpapers.
# Wallpaper Engine 壁纸下载路径。
# Default/默认: ~/.local/share/Steam/steamapps/workshop/content/431960
search_path = "~/.local/share/Steam/steamapps/workshop/content/431960"

# The base output path for extracted wallpapers.
# 提取壁纸的基础输出路径。
# Default/默认: ~/.local/share/lianpkg/
output_path = "~/.local/share/lianpkg/"

# Optional: Separate path for storing extracted video files (.mp4).
# 可选：用于存储提取的视频文件 (.mp4) 的单独路径。
# If not set, videos are stored in output_path.
# 如果未设置，视频将存储在 output_path 中。
# video_path = "/path/to/videos"

[pkg]
# The output path for unpacked PKG files.
# 解包 PKG 文件的输出路径。
# Default/默认: ~/.local/share/lianpkg/Pkg_converted
output_path = "~/.local/share/lianpkg/Pkg_converted"
"#;
        if let Err(e) = fs::write(&default_path, default_config_content) {
            return ConfigStatus::Error(format!("Failed to write default config: {}", e));
        }
        return ConfigStatus::CreatedDefault(default_path);
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
