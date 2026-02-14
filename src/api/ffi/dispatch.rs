//! FFI 调度逻辑 - 将 action 分发到对应的 API 函数

use super::types::*;
use crate::api::native::{auto, context, pkg, scan, tex};
use crate::core::cfg::{self, ProcessType};
use std::path::PathBuf;

/// 分发请求到对应的处理函数
pub fn dispatch(request: FfiRequest) -> FfiResponse {
    match request.action.as_str() {
        "init" => handle_init(request.params),
        "scan" => handle_scan(request.params),
        "auto" => handle_auto(request.params),
        "progress" => handle_progress(),
        "pkg_unpack" => handle_pkg_unpack(request.params),
        "pkg_preview" => handle_pkg_preview(request.params),
        "tex_convert" => handle_tex_convert(request.params),
        "tex_preview" => handle_tex_preview(request.params),
        "config_get" => handle_config_get(),
        "config_set" => handle_config_set(request.params),
        "config_reset" => handle_config_reset(),
        "state_get" => handle_state_get(),
        "state_clear" => handle_state_clear(),
        "status" => handle_status(),
        _ => FfiResponse::error(format!("Unknown action: {}", request.action)),
    }
}

// ============================================================================
// 辅助：获取上下文
// ============================================================================

fn get_ctx() -> Result<context::AppContext, FfiResponse> {
    context::init(None).map_err(|e| FfiResponse::error(format!("Failed to init context: {}", e)))
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

    let ctx = match get_ctx() {
        Ok(c) => c,
        Err(r) => return r,
    };

    let workshop_path = params
        .workshop_path
        .map(PathBuf::from)
        .unwrap_or(ctx.config.workshop_path.clone());

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

    let mut ctx = match get_ctx() {
        Ok(c) => c,
        Err(r) => return r,
    };

    // 应用 FFI 参数覆盖
    if params.no_raw {
        ctx.config.enable_raw_output = false;
    }
    if params.no_tex {
        ctx.config.pipeline.auto_convert_tex = false;
    }
    if params.no_clean_unpacked {
        ctx.config.clean_unpacked = false;
    }
    if params.no_incremental {
        ctx.config.pipeline.incremental = false;
    }

    // 设置进度回调，将进度写入全局状态
    PROGRESS.reset();
    PROGRESS.start();

    let progress_cb = |p: auto::AutoProgress| {
        let stage = match p.stage {
            auto::AutoStage::Init => "init",
            auto::AutoStage::Scanning => "scanning",
            auto::AutoStage::Copying => "copying",
            auto::AutoStage::Unpacking => "unpacking",
            auto::AutoStage::Converting => "converting",
            auto::AutoStage::Cleanup => "cleanup",
            auto::AutoStage::Done => "done",
        };
        PROGRESS.update(stage, p.progress, &p.message, p.current_item);
    };

    let options = auto::AutoOptions {
        wallpaper_ids: params.wallpaper_ids,
        progress: Some(&progress_cb),
    };

    let result = auto::run_auto(&ctx, options);

    PROGRESS.finish();

    match result {
        Ok(output) => FfiResponse::success(&output),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_progress() -> FfiResponse {
    FfiResponse::success(&PROGRESS.snapshot())
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

fn handle_pkg_preview(params: serde_json::Value) -> FfiResponse {
    let params: PkgPreviewParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return FfiResponse::error(format!("Invalid params for pkg_preview: {}", e)),
    };

    let path = PathBuf::from(params.path);

    match pkg::preview_pkg(&path) {
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
        None => match get_ctx() {
            Ok(ctx) => ctx.config.converted_output_path.clone(),
            Err(r) => return r,
        },
    };

    match tex::convert_all(&input, &output) {
        Ok(result) => FfiResponse::success(&result),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_tex_preview(params: serde_json::Value) -> FfiResponse {
    let params: TexPreviewParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return FfiResponse::error(format!("Invalid params for tex_preview: {}", e)),
    };

    let path = PathBuf::from(params.path);

    match tex::preview_tex(&path) {
        Ok(result) => FfiResponse::success(&result),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_config_get() -> FfiResponse {
    match get_ctx() {
        Ok(ctx) => FfiResponse::success(&ctx.config),
        Err(r) => r,
    }
}

fn handle_config_set(params: serde_json::Value) -> FfiResponse {
    let params: ConfigSetParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => return FfiResponse::error(format!("Invalid params for config_set: {}", e)),
    };

    let ctx = match get_ctx() {
        Ok(c) => c,
        Err(r) => return r,
    };

    match cfg::update_config_toml(cfg::UpdateConfigInput {
        path: ctx.config_path,
        key: params.key,
        value: params.value,
    }) {
        Ok(_) => FfiResponse::success(&serde_json::json!({})),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_config_reset() -> FfiResponse {
    let ctx = match get_ctx() {
        Ok(c) => c,
        Err(r) => return r,
    };

    // 删除后重建
    let _ = cfg::delete_config_toml(cfg::DeleteConfigInput {
        path: ctx.config_path.clone(),
    });

    match cfg::create_config_toml(cfg::CreateConfigInput {
        path: ctx.config_path,
        content: None,
    }) {
        Ok(_) => FfiResponse::success(&serde_json::json!({})),
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_state_get() -> FfiResponse {
    let ctx = match get_ctx() {
        Ok(c) => c,
        Err(r) => return r,
    };

    let state = context::load_state_or_default(&ctx.state_path);
    FfiResponse::success(&state)
}

fn handle_state_clear() -> FfiResponse {
    let ctx = match get_ctx() {
        Ok(c) => c,
        Err(r) => return r,
    };

    match cfg::delete_state_json(cfg::DeleteStateInput {
        path: ctx.state_path.clone(),
    }) {
        Ok(_) => {
            // 重建空状态
            let _ = cfg::create_state_json(cfg::CreateStateInput {
                path: ctx.state_path,
                content: None,
            });
            FfiResponse::success(&serde_json::json!({}))
        }
        Err(e) => FfiResponse::error(e.to_string()),
    }
}

fn handle_status() -> FfiResponse {
    let ctx = match get_ctx() {
        Ok(c) => c,
        Err(r) => return r,
    };

    let state = context::load_state_or_default(&ctx.state_path);

    // 扫描当前壁纸目录，与 state 交叉对比
    let scan_result = match scan::scan_workshop(&ctx.config.workshop_path, Some(&state)) {
        Ok(r) => r,
        Err(e) => return FfiResponse::error(format!("Scan failed: {}", e)),
    };

    // 已处理统计（来自 state）
    let mut processed_pkg = 0usize;
    let mut processed_raw = 0usize;
    let mut processed_skipped = 0usize;
    for info in state.processed.values() {
        match info.process_type {
            ProcessType::Pkg | ProcessType::PkgTex => processed_pkg += 1,
            ProcessType::Raw => processed_raw += 1,
            ProcessType::Skipped => processed_skipped += 1,
        }
    }

    // 待处理统计（扫描结果中未处理的）
    let mut pending_pkg = 0usize;
    let mut pending_raw = 0usize;
    let mut pending_pkg_size: u64 = 0;
    for w in &scan_result.wallpapers {
        if !w.is_processed {
            if w.has_pkg {
                pending_pkg += 1;
                for pkg_path in &w.pkg_files {
                    if let Ok(meta) = std::fs::metadata(pkg_path) {
                        pending_pkg_size += meta.len();
                    }
                }
            } else {
                pending_raw += 1;
            }
        }
    }

    // 实际输出目录大小
    let raw_output_size = crate::core::disk::get_dir_size(&ctx.config.raw_output_path);
    let unpacked_output_size = crate::core::disk::get_dir_size(&ctx.config.unpacked_output_path);
    let converted_output_size = crate::core::disk::get_dir_size(&ctx.config.converted_output_path);

    // 磁盘可用空间
    let available_space = crate::core::disk::check_space(
        crate::core::disk::CheckSpaceInput {
            path: ctx.config.converted_output_path.clone(),
        },
    )
    .ok()
    .map(|s| s.available);

    FfiResponse::success(&serde_json::json!({
        "total_wallpapers": scan_result.stats.total_count,
        "total_processed": state.processed.len(),
        "processed_pkg": processed_pkg,
        "processed_raw": processed_raw,
        "processed_skipped": processed_skipped,
        "pending_total": pending_pkg + pending_raw,
        "pending_pkg": pending_pkg,
        "pending_raw": pending_raw,
        "pending_pkg_size": pending_pkg_size,
        "last_run": state.last_run,
        "disk_usage": {
            "raw_output_size": raw_output_size,
            "unpacked_output_size": unpacked_output_size,
            "converted_output_size": converted_output_size,
            "available_space": available_space,
        },
    }))
}
