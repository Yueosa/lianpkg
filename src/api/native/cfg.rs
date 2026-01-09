//! 配置管理高级接口
//!
//! 提供初始化、解析、保存等配置相关的便捷方法。
//! 封装 core::cfg 的底层操作，提供更友好的 API。

use crate::core::{cfg, path};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// 结构体定义
// ============================================================================

/// 初始化配置入参
#[derive(Debug, Clone)]
pub struct InitConfigInput {
    /// 自定义配置目录，None 则使用默认目录
    pub config_dir: Option<PathBuf>,
    /// 是否优先使用 exe 同目录（仅 Windows）
    pub use_exe_dir: bool,
}

/// 初始化配置返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitConfigOutput {
    /// 是否成功
    pub success: bool,
    /// config.toml 是否新创建
    pub config_created: bool,
    /// state.json 是否新创建
    pub state_created: bool,
    /// config.toml 路径
    pub config_path: PathBuf,
    /// state.json 路径
    pub state_path: PathBuf,
    /// 错误信息
    pub error: Option<String>,
}

/// 解析后的运行时配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Workshop 路径
    pub workshop_path: PathBuf,
    /// 原始壁纸输出路径
    pub raw_output_path: PathBuf,
    /// 是否启用原始壁纸输出
    pub enable_raw_output: bool,
    /// Pkg 临时路径
    pub pkg_temp_path: PathBuf,
    /// 解包输出路径
    pub unpacked_output_path: PathBuf,
    /// 是否清理 pkg_temp
    pub clean_pkg_temp: bool,
    /// 是否清理 unpacked
    pub clean_unpacked: bool,
    /// Tex 转换输出路径（可选）
    pub converted_output_path: Option<PathBuf>,
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

/// 加载配置入参
#[derive(Debug, Clone)]
pub struct LoadConfigInput {
    /// config.toml 路径
    pub config_path: PathBuf,
}

/// 加载配置返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadConfigOutput {
    /// 是否成功
    pub success: bool,
    /// 解析后的配置
    pub config: Option<RuntimeConfig>,
    /// 错误信息
    pub error: Option<String>,
}

/// 加载状态入参
#[derive(Debug, Clone)]
pub struct LoadStateInput {
    /// state.json 路径
    pub state_path: PathBuf,
}

/// 加载状态返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadStateOutput {
    /// 是否成功
    pub success: bool,
    /// 解析后的状态数据
    pub state: Option<cfg::StateData>,
    /// 错误信息
    pub error: Option<String>,
}

/// 保存状态入参
#[derive(Debug, Clone)]
pub struct SaveStateInput {
    /// state.json 路径
    pub state_path: PathBuf,
    /// 状态数据
    pub state: cfg::StateData,
}

/// 保存状态返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveStateOutput {
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
}

// ============================================================================
// 接口实现
// ============================================================================

/// 初始化配置文件
///
/// 确保 config.toml 和 state.json 都存在，不存在则创建默认内容
pub fn init_config(input: InitConfigInput) -> InitConfigOutput {
    // 确定配置目录
    let config_dir = input.config_dir.unwrap_or_else(|| {
        // Windows 下，如果设置了 use_exe_dir，优先使用 exe 同目录
        #[cfg(target_os = "windows")]
        {
            if input.use_exe_dir {
                if let Some(dir) = path::exe_config_dir() {
                    return dir;
                }
            }
        }
        path::default_config_dir()
    });
    let config_path = config_dir.join("config.toml");
    let state_path = config_dir.join("state.json");

    // 创建 config.toml
    let config_result = cfg::create_config_toml(cfg::CreateConfigInput {
        path: config_path.clone(),
        content: None,
    });

    // 创建 state.json
    let state_result = cfg::create_state_json(cfg::CreateStateInput {
        path: state_path.clone(),
        content: None,
    });

    // 检查结果
    let (config_created, state_created) = match (config_result, state_result) {
        (Ok(c), Ok(s)) => (c.created, s.created),
        (Ok(c), Err(_)) => (c.created, false),
        (Err(_), Ok(s)) => (false, s.created),
        (Err(_), Err(_)) => (false, false),
    };

    InitConfigOutput {
        success: true,
        config_created,
        state_created,
        config_path,
        state_path,
        error: None,
    }
}

/// 加载并解析 config.toml
///
/// 将 TOML 配置文件解析为 RuntimeConfig 结构
pub fn load_config(input: LoadConfigInput) -> LoadConfigOutput {
    // 读取文件
    let read_result = cfg::read_config_toml(cfg::ReadConfigInput {
        path: input.config_path,
    });

    let content = match read_result {
        Ok(r) => r.content,
        Err(e) => {
            return LoadConfigOutput {
                success: false,
                config: None,
                error: Some(format!("Failed to read config.toml: {}", e)),
            };
        }
    };

    // 解析 TOML
    match parse_config_toml(&content) {
        Ok(config) => LoadConfigOutput {
            success: true,
            config: Some(config),
            error: None,
        },
        Err(e) => LoadConfigOutput {
            success: false,
            config: None,
            error: Some(e),
        },
    }
}

/// 加载并解析 state.json
pub fn load_state(input: LoadStateInput) -> LoadStateOutput {
    let read_result = cfg::read_state_json(cfg::ReadStateInput {
        path: input.state_path,
    });

    let content = match read_result {
        Ok(r) => r.content,
        Err(e) => {
            return LoadStateOutput {
                success: false,
                state: None,
                error: Some(format!("Failed to read state.json: {}", e)),
            };
        }
    };

    match serde_json::from_str::<cfg::StateData>(&content) {
        Ok(state) => LoadStateOutput {
            success: true,
            state: Some(state),
            error: None,
        },
        Err(e) => LoadStateOutput {
            success: false,
            state: None,
            error: Some(format!("Failed to parse state.json: {}", e)),
        },
    }
}

/// 保存 state.json
pub fn save_state(input: SaveStateInput) -> SaveStateOutput {
    let content = match serde_json::to_string_pretty(&input.state) {
        Ok(c) => c,
        Err(e) => {
            return SaveStateOutput {
                success: false,
                error: Some(format!("Failed to serialize state: {}", e)),
            };
        }
    };

    let write_result = cfg::write_state_json(cfg::WriteStateInput {
        path: input.state_path,
        content,
    });

    match write_result {
        Ok(_) => SaveStateOutput {
            success: true,
            error: None,
        },
        Err(e) => SaveStateOutput {
            success: false,
            error: Some(format!("Failed to write state.json: {}", e)),
        },
    }
}

/// 检查壁纸是否已处理
pub fn is_wallpaper_processed(state: &cfg::StateData, wallpaper_id: &str) -> bool {
    state
        .processed_wallpapers
        .iter()
        .any(|w| w.wallpaper_id == wallpaper_id)
}

/// 添加已处理壁纸记录
pub fn add_processed_wallpaper(
    state: &mut cfg::StateData,
    wallpaper_id: String,
    title: Option<String>,
    process_type: cfg::WallpaperProcessType,
    output_path: Option<String>,
) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    state.processed_wallpapers.push(cfg::ProcessedWallpaper {
        wallpaper_id,
        title,
        process_type,
        processed_at: now,
        output_path,
    });
}

/// 更新统计信息
pub fn update_statistics(state: &mut cfg::StateData, wallpapers: u64, pkgs: u64, texs: u64) {
    use std::time::{SystemTime, UNIX_EPOCH};

    state.statistics.total_runs += 1;
    state.statistics.total_wallpapers += wallpapers;
    state.statistics.total_pkgs += pkgs;
    state.statistics.total_texs += texs;

    state.last_run = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    );
}

// ============================================================================
// 内部工具函数
// ============================================================================

/// 解析 config.toml 内容为 RuntimeConfig
fn parse_config_toml(content: &str) -> Result<RuntimeConfig, String> {
    let doc: toml::Table =
        toml::from_str(content).map_err(|e| format!("TOML parse error: {}", e))?;

    // 解析 [wallpaper] 部分
    let wallpaper = doc
        .get("wallpaper")
        .and_then(|v| v.as_table())
        .ok_or("Missing [wallpaper] section")?;

    let workshop_path = wallpaper
        .get("workshop_path")
        .and_then(|v| v.as_str())
        .map(path::expand_path_compat)
        .unwrap_or_else(|| PathBuf::from(path::default_workshop_path()));

    let raw_output_path = wallpaper
        .get("raw_output_path")
        .and_then(|v| v.as_str())
        .map(path::expand_path_compat)
        .unwrap_or_else(|| PathBuf::from(path::default_raw_output_path()));

    let enable_raw_output = wallpaper
        .get("enable_raw_output")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let pkg_temp_path = wallpaper
        .get("pkg_temp_path")
        .and_then(|v| v.as_str())
        .map(path::expand_path_compat)
        .unwrap_or_else(|| PathBuf::from(path::default_pkg_temp_path()));

    // 解析 [unpack] 部分
    let unpack = doc.get("unpack").and_then(|v| v.as_table());

    let unpacked_output_path = unpack
        .and_then(|u| u.get("unpacked_output_path"))
        .and_then(|v| v.as_str())
        .map(path::expand_path_compat)
        .unwrap_or_else(|| PathBuf::from(path::default_unpacked_output_path()));

    let clean_pkg_temp = unpack
        .and_then(|u| u.get("clean_pkg_temp"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

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
        .map(path::expand_path_compat);

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
        pkg_temp_path,
        unpacked_output_path,
        clean_pkg_temp,
        clean_unpacked,
        converted_output_path,
        pipeline,
    })
}
