//! FFI 调度逻辑 - 将 action 分发到对应的 API 函数

use super::types::*;
use crate::api::native::{auto, context, pkg, scan, tex};
use std::path::PathBuf;

/// 分发请求到对应的处理函数
pub fn dispatch(request: FfiRequest) -> FfiResponse {
    match request.action.as_str() {
        "init" => handle_init(request.params),
        "scan" => handle_scan(request.params),
        "auto" => handle_auto(request.params),
        "pkg_unpack" => handle_pkg_unpack(request.params),
        "tex_convert" => handle_tex_convert(request.params),
        "config_get" => handle_config_get(),
        "config_set" => handle_config_set(request.params),
        "status" => handle_status(),
        _ => FfiResponse::error(format!("Unknown action: {}", request.action)),
    }
}

// ============================================================================
// Action 处理函数
// ============================================================================

fn handle_init(params: serde_json::Value) -> FfiResponse {
    let params: InitParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return FfiResponse::error(format!("Invalid params for init: {}", e)),
    };

    let config_dir = params.config_dir.map(PathBuf::from);
    
    match context::init(config_dir) {
        Ok(ctx) => FfiResponse::success(&ctx),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_scan(params: serde_json::Value) -> FfiResponse {
    let params: ScanParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return FfiResponse::error(format!("Invalid params for scan: {}", e)),
    };

    // 需要先初始化 context 获取配置
    let ctx = match context::init(None) {
        Ok(c) => c,
        Err(e) => return FfiResponse::error(format!("Failed to init context: {}", e)),
    };

    let workshop_path = params
        .workshop_path
        .map(PathBuf::from)
        .unwrap_or(ctx.config.workshop_path.clone());

    // 加载状态（用于增量处理）
    let state = context::load_state_or_default(&ctx.state_path);

    match scan::scan_workshop(&workshop_path, Some(&state)) {
        Ok(result) => FfiResponse::success(&result),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_auto(params: serde_json::Value) -> FfiResponse {
    let params: AutoParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return FfiResponse::error(format!("Invalid params for auto: {}", e)),
    };

    let ctx = match context::init(None) {
        Ok(c) => c,
        Err(e) => return FfiResponse::error(format!("Failed to init context: {}", e)),
    };

    let options = auto::AutoOptions {
        wallpaper_ids: params.wallpaper_ids,
        progress: None, // FFI 暂不支持进度回调
    };

    match auto::run_auto(&ctx, options) {
        Ok(result) => FfiResponse::success(&result),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_pkg_unpack(params: serde_json::Value) -> FfiResponse {
    let params: PkgUnpackParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return FfiResponse::error(format!("Invalid params for pkg_unpack: {}", e)),
    };

    let sources: Vec<pkg::PkgSource> = params
        .sources
        .into_iter()
        .map(|s| pkg::PkgSource {
            wallpaper_id: s.wallpaper_id,
            pkg_paths: s.pkg_paths.into_iter().map(PathBuf::from).collect(),
        })
        .collect();

    let output = PathBuf::from(params.output);

    match pkg::unpack_all(&sources, &output) {
        Ok(result) => FfiResponse::success(&result),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_tex_convert(params: serde_json::Value) -> FfiResponse {
    let params: TexConvertParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return FfiResponse::error(format!("Invalid params for tex_convert: {}", e)),
    };

    let input = PathBuf::from(params.input);
    let output = match params.output {
        Some(ref s) => PathBuf::from(s),
        None => {
            // 使用配置中的默认 converted_output_path
            match context::init(None) {
                Ok(ctx) => ctx.config.converted_output_path.clone(),
                Err(e) => return FfiResponse::error(format!("No output specified and failed to load config: {}", e)),
            }
        }
    };

    match tex::convert_all(&input, &output) {
        Ok(result) => FfiResponse::success(&result),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_config_get() -> FfiResponse {
    let ctx = match context::init(None) {
        Ok(c) => c,
        Err(e) => return FfiResponse::error(format!("Failed to init context: {}", e)),
    };

    FfiResponse::success(&ctx.config)
}

fn handle_config_set(params: serde_json::Value) -> FfiResponse {
    let params: ConfigSetParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return FfiResponse::error(format!("Invalid params for config_set: {}", e)),
    };

    let ctx = match context::init(None) {
        Ok(c) => c,
        Err(e) => return FfiResponse::error(format!("Failed to init context: {}", e)),
    };

    // 使用 cfg API 更新配置
    use crate::core::cfg;
    match cfg::update_config_toml(cfg::UpdateConfigInput {
        path: ctx.config_path,
        key: params.key,
        value: params.value,
    }) {
        Ok(_) => FfiResponse::success(&serde_json::json!({})),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_status() -> FfiResponse {
    let ctx = match context::init(None) {
        Ok(c) => c,
        Err(e) => return FfiResponse::error(format!("Failed to init context: {}", e)),
    };

    let state = context::load_state_or_default(&ctx.state_path);
    FfiResponse::success(&state)
}
