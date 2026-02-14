//! FFI 类型定义 - 请求/响应结构 + 进度状态

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;

// ============================================================================
// 通用请求/响应
// ============================================================================

/// FFI 请求
#[derive(Debug, Deserialize)]
pub struct FfiRequest {
    /// 操作类型
    pub action: String,
    /// 操作参数（JSON 对象）
    #[serde(default)]
    pub params: serde_json::Value,
}

/// FFI 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct FfiResponse {
    /// 是否成功
    pub success: bool,
    /// 成功时的数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// 失败时的错误消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl FfiResponse {
    /// 成功响应
    pub fn success(data: impl Serialize) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).ok(),
            error: None,
        }
    }

    /// 错误响应
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

// ============================================================================
// 进度状态（全局共享，用于轮询）
// ============================================================================

/// 全局进度状态
pub static PROGRESS: GlobalProgress = GlobalProgress::new();

pub struct GlobalProgress {
    pub running: AtomicBool,
    pub percent: AtomicU8,
    pub stage: Mutex<String>,
    pub message: Mutex<String>,
    pub current_item: Mutex<Option<String>>,
}

impl GlobalProgress {
    const fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            percent: AtomicU8::new(0),
            stage: Mutex::new(String::new()),
            message: Mutex::new(String::new()),
            current_item: Mutex::new(None),
        }
    }

    pub fn reset(&self) {
        self.running.store(false, Ordering::Relaxed);
        self.percent.store(0, Ordering::Relaxed);
        *self.stage.lock().unwrap() = String::new();
        *self.message.lock().unwrap() = String::new();
        *self.current_item.lock().unwrap() = None;
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::Relaxed);
        self.percent.store(0, Ordering::Relaxed);
    }

    pub fn finish(&self) {
        self.running.store(false, Ordering::Relaxed);
        self.percent.store(100, Ordering::Relaxed);
    }

    pub fn update(&self, stage: &str, percent: u8, message: &str, item: Option<String>) {
        self.percent.store(percent, Ordering::Relaxed);
        *self.stage.lock().unwrap() = stage.to_string();
        *self.message.lock().unwrap() = message.to_string();
        *self.current_item.lock().unwrap() = item;
    }

    pub fn snapshot(&self) -> ProgressSnapshot {
        ProgressSnapshot {
            running: self.running.load(Ordering::Relaxed),
            percent: self.percent.load(Ordering::Relaxed),
            stage: self.stage.lock().unwrap().clone(),
            message: self.message.lock().unwrap().clone(),
            current_item: self.current_item.lock().unwrap().clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProgressSnapshot {
    pub running: bool,
    pub percent: u8,
    pub stage: String,
    pub message: String,
    pub current_item: Option<String>,
}

// ============================================================================
// 各 Action 的参数定义
// ============================================================================

/// init 参数
#[derive(Debug, Deserialize)]
pub struct InitParams {
    pub config_dir: Option<String>,
}

/// scan 参数
#[derive(Debug, Deserialize)]
pub struct ScanParams {
    pub workshop_path: Option<String>,
}

/// auto 参数
#[derive(Debug, Deserialize)]
pub struct AutoParams {
    pub wallpaper_ids: Option<Vec<String>>,
    #[serde(default)]
    pub no_raw: bool,
    #[serde(default)]
    pub no_tex: bool,
    #[serde(default)]
    pub no_clean_unpacked: bool,
    #[serde(default)]
    pub no_incremental: bool,
}

/// pkg_unpack 参数
#[derive(Debug, Deserialize)]
pub struct PkgUnpackParams {
    pub sources: Vec<PkgSourceDto>,
    pub output: String,
}

#[derive(Debug, Deserialize)]
pub struct PkgSourceDto {
    pub wallpaper_id: String,
    pub pkg_paths: Vec<String>,
}

/// pkg_preview 参数
#[derive(Debug, Deserialize)]
pub struct PkgPreviewParams {
    pub path: String,
}

/// tex_convert 参数
#[derive(Debug, Deserialize)]
pub struct TexConvertParams {
    pub input: String,
    pub output: Option<String>,
}

/// tex_preview 参数
#[derive(Debug, Deserialize)]
pub struct TexPreviewParams {
    pub path: String,
}

/// config_set 参数
#[derive(Debug, Deserialize)]
pub struct ConfigSetParams {
    pub key: String,
    pub value: String,
}
