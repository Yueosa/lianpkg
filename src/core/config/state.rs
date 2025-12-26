use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::config::utl::{ensure_dir, resolve_config_dir};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct State {
    /// Identifiers or paths that已处理；由上层决定语义。
    pub processed: Vec<String>,
}

pub fn state_file_path(custom_path: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(p) = custom_path {
        return Ok(p);
    }
    Ok(resolve_config_dir()?.join("state.json"))
}

pub fn load_state(custom_path: Option<PathBuf>) -> Result<State, String> {
    let path = state_file_path(custom_path)?;
    if !path.exists() {
        return Ok(State::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read state {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse state {}: {}", path.display(), e))
}

pub fn save_state(custom_path: Option<PathBuf>, state: &State) -> Result<(), String> {
    let path = state_file_path(custom_path)?;
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write state {}: {}", path.display(), e))
}

pub fn delete_state(custom_path: Option<PathBuf>) -> Result<(), String> {
    let path = state_file_path(custom_path)?;
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete state {}: {}", path.display(), e))?;
    Ok(())
}

pub fn mark_processed(state: &mut State, item: impl Into<String>) -> bool {
    let item_str = item.into();
    let mut set: HashSet<String> = state.processed.iter().cloned().collect();
    let inserted = set.insert(item_str.clone());
    if inserted {
        state.processed = set.into_iter().collect();
    }
    inserted
}

pub fn clear_state(state: &mut State) {
    state.processed.clear();
}
