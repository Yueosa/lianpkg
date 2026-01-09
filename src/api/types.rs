use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum StatusCode {
    Success,
    Warning,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResult<T> {
    pub status: StatusCode,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WallpaperTaskResult {
    pub raw_count: usize,
    pub pkg_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PkgTaskResult {
    pub processed_files: usize,
    pub extracted_files: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TexTaskResult {
    pub processed_files: usize,
    pub converted_files: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoTaskResult {
    pub wallpaper: WallpaperTaskResult,
    pub pkg: PkgTaskResult,
    pub tex: TexTaskResult,
}
