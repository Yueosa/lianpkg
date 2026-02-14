//! 应用上下文与配置管理
//!
//! 提供应用初始化、配置加载/保存等核心功能。
//! `AppContext` 是所有 API 调用的基础上下文。

use crate::core::{cfg, error::{CoreError, CoreResult}, path};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// 核心类型
// ============================================================================

/// 应用上下文
///
/// 在 `init()` 中一次性构建，包含运行时配置和文件路径。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    /// 运行时配置（从 config.toml 解析）
    pub config: RuntimeConfig,
    /// config.toml 路径
    pub config_path: PathBuf,
    /// state.json 路径
    pub state_path: PathBuf,
}

/// 运行时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Workshop 路径
    pub workshop_path: PathBuf,
    /// 原始壁纸输出路径
    pub raw_output_path: PathBuf,
    /// 是否启用原始壁纸输出
    pub enable_raw_output: bool,
    /// 解包输出路径
    pub unpacked_output_path: PathBuf,
    /// 是否清理 unpacked
    pub clean_unpacked: bool,
    /// Tex 转换输出路径
    pub converted_output_path: PathBuf,
    /// 流水线配置
    pub pipeline: PipelineConfig,
}

/// 流水线配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PipelineConfig {
    /// 是否增量处理
    pub incremental: bool,
    /// 是否自动解包 pkg
    pub auto_unpack_pkg: bool,
    /// 是否自动转换 tex
    pub auto_convert_tex: bool,
}

// ============================================================================
// 初始化
// ============================================================================

/// 初始化应用上下文
///
/// 1. 确定配置目录（支持自定义路径和 Windows exe 同目录模式）
/// 2. 确保 config.toml / state.json 存在
/// 3. 加载并解析配置
/// 4. 返回 AppContext
pub fn init(config_dir: Option<PathBuf>) -> CoreResult<AppContext> {
    let config_dir = config_dir.unwrap_or_else(|| path::default_config_dir());
    let config_path = config_dir.join("config.toml");
    let state_path = config_dir.join("state.json");

    // 确保配置文件存在
    cfg::create_config_toml(cfg::CreateConfigInput {
        path: config_path.clone(),
        content: None,
    })?;
    cfg::create_state_json(cfg::CreateStateInput {
        path: state_path.clone(),
        content: None,
    })?;

    // 加载配置
    let config = load_config(&config_path)?;

    Ok(AppContext {
        config,
        config_path,
        state_path,
    })
}

/// Windows 专用：优先使用 exe 同目录的初始化
///
/// 当 `config_dir` 为 None 且在 Windows 上运行时，
/// 尝试使用 exe 所在目录作为配置目录。
#[cfg(target_os = "windows")]
pub fn init_with_exe_dir(config_dir: Option<PathBuf>) -> CoreResult<AppContext> {
    let dir = config_dir.or_else(|| path::exe_config_dir());
    init(dir)
}

// ============================================================================
// 配置加载与保存
// ============================================================================

/// 加载并解析 config.toml
pub fn load_config(config_path: &std::path::Path) -> CoreResult<RuntimeConfig> {
    let result = cfg::read_config_toml(cfg::ReadConfigInput {
        path: config_path.to_path_buf(),
    })?;
    parse_config_toml(&result.content)
}

/// 加载 state.json
pub fn load_state(state_path: &std::path::Path) -> CoreResult<cfg::StateData> {
    let result = cfg::read_state_json(cfg::ReadStateInput {
        path: state_path.to_path_buf(),
    })?;
    serde_json::from_str::<cfg::StateData>(&result.content)
        .map_err(|e| CoreError::parse_with_source(e.to_string(), "state.json"))
}

/// 加载状态，失败时返回默认空状态
pub fn load_state_or_default(state_path: &std::path::Path) -> cfg::StateData {
    load_state(state_path).unwrap_or_default()
}

/// 保存 state.json
pub fn save_state(state_path: &std::path::Path, state: &cfg::StateData) -> CoreResult<()> {
    let content = serde_json::to_string_pretty(state)?;
    cfg::write_state_json(cfg::WriteStateInput {
        path: state_path.to_path_buf(),
        content,
    })?;
    Ok(())
}

// ============================================================================
// 状态操作辅助函数
// ============================================================================

/// 检查壁纸是否已处理（HashMap O(1) 查找）
pub fn is_wallpaper_processed(state: &cfg::StateData, wallpaper_id: &str) -> bool {
    state.processed.contains_key(wallpaper_id)
}

/// 添加已处理壁纸记录
pub fn add_processed_wallpaper(
    state: &mut cfg::StateData,
    wallpaper_id: String,
    title: Option<String>,
    process_type: cfg::ProcessType,
    output_path: Option<String>,
) {
    let now = chrono::Utc::now().to_rfc3339();
    state.processed.insert(
        wallpaper_id,
        cfg::ProcessedEntry {
            title,
            process_type,
            processed_at: now,
            output_path,
        },
    );
}

/// 更新 last_run 时间戳（ISO 8601）
pub fn touch_last_run(state: &mut cfg::StateData) {
    state.last_run = Some(chrono::Utc::now().to_rfc3339());
}

// ============================================================================
// 内部：解析 config.toml
// ============================================================================

fn parse_config_toml(content: &str) -> CoreResult<RuntimeConfig> {
    let doc: toml::Table = toml::from_str(content)?;

    // 解析 [wallpaper] 部分
    let wallpaper = doc
        .get("wallpaper")
        .and_then(|v| v.as_table())
        .ok_or_else(|| CoreError::validation("Missing [wallpaper] section in config.toml"))?;

    let workshop_path = wallpaper
        .get("workshop_path")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(path::expand_path_compat)
        .or_else(|| path::detect_workshop_path().ok())
        .unwrap_or_else(|| PathBuf::from("/nonexistent/workshop"));

    let raw_output_path = wallpaper
        .get("raw_output_path")
        .and_then(|v| v.as_str())
        .map(path::expand_path_compat)
        .unwrap_or_else(|| PathBuf::from(path::default_raw_output_path()));

    let enable_raw_output = wallpaper
        .get("enable_raw_output")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // 解析 [unpack] 部分
    let unpack = doc.get("unpack").and_then(|v| v.as_table());

    let unpacked_output_path = unpack
        .and_then(|u| u.get("unpacked_output_path"))
        .and_then(|v| v.as_str())
        .map(path::expand_path_compat)
        .unwrap_or_else(|| PathBuf::from(path::default_unpacked_output_path()));

    let clean_unpacked = unpack
        .and_then(|u| u.get("clean_unpacked"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // 解析 [tex] 部分
    let tex = doc.get("tex").and_then(|v| v.as_table());

    let converted_output_path = tex
        .and_then(|t| t.get("converted_output_path"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(path::expand_path_compat)
        .unwrap_or_else(|| PathBuf::from(path::default_converted_output_path()));

    // 解析 [pipeline] 部分
    let pipeline_section = doc.get("pipeline").and_then(|v| v.as_table());

    let pipeline = PipelineConfig {
        incremental: pipeline_section
            .and_then(|p| p.get("incremental"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        auto_unpack_pkg: pipeline_section
            .and_then(|p| p.get("auto_unpack_pkg"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        auto_convert_tex: pipeline_section
            .and_then(|p| p.get("auto_convert_tex"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    };

    Ok(RuntimeConfig {
        workshop_path,
        raw_output_path,
        enable_raw_output,
        unpacked_output_path,
        clean_unpacked,
        converted_output_path,
        pipeline,
    })
}
