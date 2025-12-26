use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WallpaperStats {
    pub raw_count: usize,
    pub pkg_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProjectMeta {
    pub contentrating: Option<String>,
    pub description: Option<String>,
    pub file: Option<String>,
    pub preview: Option<String>,
    pub tags: Option<Vec<String>>,
    pub title: Option<String>,
    pub r#type: Option<String>,
    pub version: Option<u32>,
    pub workshopid: Option<String>,
    pub workshopurl: Option<String>,
    #[serde(default)]
    pub general: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FolderProcess {
    pub copied_raw: bool,
    pub copied_pkgs: usize,
}
