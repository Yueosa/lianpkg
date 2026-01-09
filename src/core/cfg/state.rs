//! state.json 文件操作接口

use std::fs;

use crate::core::cfg::structs::{
    CreateStateInput, CreateStateOutput, DeleteStateInput, DeleteStateOutput, ReadStateInput,
    ReadStateOutput, WriteStateInput, WriteStateOutput,
};
use crate::core::cfg::utl::{default_state_template, ensure_dir};
use crate::core::error::{CoreError, CoreResult};

/// 创建状态文件
/// 如果文件已存在则不创建，返回 created = false
/// 如果不提供内容则使用默认模板 "{}"
pub fn create_state_json(input: CreateStateInput) -> CoreResult<CreateStateOutput> {
    let path = input.path;

    // 文件已存在，不触发创建
    if path.exists() {
        return Ok(CreateStateOutput {
            created: false,
            path,
        });
    }

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    // 获取内容：优先使用提供的内容，否则使用默认模板
    let content = input.content.unwrap_or_else(default_state_template);

    // 写入文件
    fs::write(&path, content)
        .map_err(|e| CoreError::io_with_path(e.to_string(), path.display().to_string()))?;

    Ok(CreateStateOutput {
        created: true,
        path,
    })
}

/// 读取状态文件
/// 返回文件内容
pub fn read_state_json(input: ReadStateInput) -> CoreResult<ReadStateOutput> {
    let path = input.path;

    if !path.exists() {
        return Err(CoreError::not_found_with_path(
            "State file not found",
            path.display().to_string(),
        ));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| CoreError::io_with_path(e.to_string(), path.display().to_string()))?;

    Ok(ReadStateOutput { content })
}

/// 覆写状态文件
/// 直接用新内容覆盖整个文件
pub fn write_state_json(input: WriteStateInput) -> CoreResult<WriteStateOutput> {
    let path = input.path.clone();
    let content = input.content;

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    // 写入文件
    fs::write(&path, &content)
        .map_err(|e| CoreError::io_with_path(e.to_string(), path.display().to_string()))?;

    Ok(WriteStateOutput { content })
}

/// 删除状态文件
/// 文件不存在视为成功，但 deleted = false
pub fn delete_state_json(input: DeleteStateInput) -> CoreResult<DeleteStateOutput> {
    let path = input.path;

    // 文件不存在，不触发删除
    if !path.exists() {
        return Ok(DeleteStateOutput {
            deleted: false,
            path,
        });
    }

    // 删除文件
    fs::remove_file(&path)
        .map_err(|e| CoreError::io_with_path(e.to_string(), path.display().to_string()))?;

    Ok(DeleteStateOutput {
        deleted: true,
        path,
    })
}
