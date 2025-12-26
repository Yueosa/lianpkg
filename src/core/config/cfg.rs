use std::fs;
use std::path::PathBuf;

use crate::core::config::structs::{Config, ConfigStatus, ConfigRaw};
use crate::core::config::utl::{
    config_file_path,
    default_raw_with_defaults,
    ensure_dir,
    build_default_config_template,
    load_raw,
    merge_raw,
    resolve_config,
    resolve_config_dir,
    write_raw,
};

/// Create config file if missing, using default resolved values as initial content.
pub fn create_config_file(custom_path: Option<PathBuf>) -> Result<PathBuf, String> {
    let path = config_file_path(custom_path)?;
    if path.exists() {
        return Ok(path);
    }
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let raw = default_raw_with_defaults();
    let tpl = build_default_config_template(&raw);
    fs::write(&path, tpl)
        .map_err(|e| format!("Failed to write default config: {}", e))?;
    Ok(path)
}

/// Load configuration; if default path missing, create it and return CreatedDefault.
pub fn load_config(custom_path: Option<PathBuf>) -> ConfigStatus {
    let path = match config_file_path(custom_path) {
        Ok(p) => p,
        Err(e) => return ConfigStatus::Error(e),
    };

    if !path.exists() {
        match create_config_file(Some(path.clone())) {
            Ok(p) => return ConfigStatus::CreatedDefault(p),
            Err(e) => return ConfigStatus::Error(e),
        }
    }

    match load_raw(&path) {
        Ok(raw) => ConfigStatus::Loaded(resolve_config(&raw)),
        Err(e) => ConfigStatus::Error(e),
    }
}

/// Delete the config file (and parent dir if empty). Succeeds if file is absent.
pub fn delete_config_file(custom_path: Option<PathBuf>) -> Result<(), String> {
    let path = config_file_path(custom_path)?;
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete config file {}: {}", path.display(), e))?;

    if let Some(dir) = path.parent() {
        if dir.read_dir().map(|mut it| it.next().is_none()).unwrap_or(false) {
            let _ = fs::remove_dir(dir);
        }
    }
    Ok(())
}

/// Delete the default config directory (legacy helper).
pub fn delete_config_dir() -> Result<(), String> {
    let config_dir = resolve_config_dir()?;

    if !config_dir.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&config_dir)
        .map_err(|e| format!("Failed to delete config dir {}: {}", config_dir.display(), e))
}

/// Update configuration by applying a partial patch, then writing back; returns resolved Config.
pub fn update_config(custom_path: Option<PathBuf>, patch: ConfigRaw) -> Result<Config, String> {
    let path = config_file_path(custom_path)?;
    if !path.exists() {
        create_config_file(Some(path.clone()))?;
    }

    let mut raw = load_raw(&path).unwrap_or_else(|_| default_raw_with_defaults());
    merge_raw(&mut raw, &patch);
    write_raw(&path, &raw)?;

    Ok(resolve_config(&raw))
}
