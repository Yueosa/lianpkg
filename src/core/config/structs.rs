use serde::{Deserialize, Serialize};
use crate::core::path;

/// Resolved configuration used at runtime (defaults applied, no Option fields).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub wallpaper: WallpaperConfig,
    pub unpack: UnpackConfig,
    pub tex: TexConfig,
}

/// Raw configuration for file IO (all fields optional for CRUD operations).
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConfigRaw {
    pub wallpaper: WallpaperConfigRaw,
    pub unpack: UnpackConfigRaw,
    pub tex: TexConfigRaw,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperConfig {
    pub workshop_path: String,
    pub raw_output_path: String,
    pub pkg_temp_path: String,
    #[serde(default = "default_true")]
    pub enable_raw_output: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WallpaperConfigRaw {
    pub workshop_path: Option<String>,
    pub raw_output_path: Option<String>,
    pub pkg_temp_path: Option<String>,
    pub enable_raw_output: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnpackConfig {
    pub unpacked_output_path: String,
    pub clean_pkg_temp: bool,
    pub clean_unpacked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UnpackConfigRaw {
    pub unpacked_output_path: Option<String>,
    pub clean_pkg_temp: Option<bool>,
    pub clean_unpacked: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TexConfig {
    pub converted_output_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TexConfigRaw {
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
                enable_raw_output: true,
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
    CreatedDefault(std::path::PathBuf),
    Error(String),
}
