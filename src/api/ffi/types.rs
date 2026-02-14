//! FFI 类型定义 - 请求/响应结构

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Serialize)]
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
// 各 Action 的参数定义
// ============================================================================

/// init 参数
#[derive(Debug, Deserialize)]
pub struct InitParams {
    /// 配置目录路径（可选，默认使用系统标准路径）
    pub config_dir: Option<String>,
}

/// scan 参数
#[derive(Debug, Deserialize)]
pub struct ScanParams {
    /// Workshop 路径（可选，默认从配置读取）
    pub workshop_path: Option<String>,
    /// 壁纸 ID 过滤（可选）
    pub ids: Option<Vec<String>>,
}

/// auto 参数
#[derive(Debug, Deserialize)]
pub struct AutoParams {
    /// 壁纸 ID 过滤（可选）
    pub wallpaper_ids: Option<Vec<String>>,
}

/// pkg_unpack 参数
#[derive(Debug, Deserialize)]
pub struct PkgUnpackParams {
    /// PKG 源列表
    pub sources: Vec<PkgSourceDto>,
    /// 输出目录
    pub output: String,
}

#[derive(Debug, Deserialize)]
pub struct PkgSourceDto {
    pub wallpaper_id: String,
    pub pkg_paths: Vec<String>,
}

/// tex_convert 参数
#[derive(Debug, Deserialize)]
pub struct TexConvertParams {
    /// 输入路径
    pub input: String,
    /// 输出路径（可选）
    pub output: Option<String>,
}

/// config_set 参数
#[derive(Debug, Deserialize)]
pub struct ConfigSetParams {
    /// 配置键
    pub key: String,
    /// 配置值
    pub value: String,
}
