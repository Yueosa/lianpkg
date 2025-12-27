//! state.json 文件操作接口

use std::fs;

use crate::core::cfg::structs::{
    CreateStateInput, CreateStateOutput,
    ReadStateInput, ReadStateOutput,
    WriteStateInput, WriteStateOutput,
    DeleteStateInput, DeleteStateOutput,
};
use crate::core::cfg::utl::{default_state_template, ensure_dir};

/// 创建状态文件
/// 如果文件已存在则不创建，返回 created = false
/// 如果不提供内容则使用默认模板 "{}"
pub fn create_state_json(input: CreateStateInput) -> CreateStateOutput {
    let path = input.path;
    
    // 文件已存在，不触发创建
    if path.exists() {
        return CreateStateOutput {
            created: false,
            path,
        };
    }
    
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        if let Err(_) = ensure_dir(parent) {
            return CreateStateOutput {
                created: false,
                path,
            };
        }
    }
    
    // 获取内容：优先使用提供的内容，否则使用默认模板
    let content = input.content.unwrap_or_else(|| default_state_template());
    
    // 写入文件
    match fs::write(&path, content) {
        Ok(_) => CreateStateOutput {
            created: true,
            path,
        },
        Err(_) => CreateStateOutput {
            created: false,
            path,
        },
    }
}

/// 读取状态文件
/// 返回文件内容，文件不存在或读取失败时 success = false
pub fn read_state_json(input: ReadStateInput) -> ReadStateOutput {
    let path = input.path;
    
    if !path.exists() {
        return ReadStateOutput {
            success: false,
            content: None,
        };
    }
    
    match fs::read_to_string(&path) {
        Ok(content) => ReadStateOutput {
            success: true,
            content: Some(content),
        },
        Err(_) => ReadStateOutput {
            success: false,
            content: None,
        },
    }
}

/// 覆写状态文件
/// 直接用新内容覆盖整个文件
pub fn write_state_json(input: WriteStateInput) -> WriteStateOutput {
    let path = input.path.clone();
    
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        if let Err(_) = ensure_dir(parent) {
            return WriteStateOutput {
                success: false,
                content: None,
            };
        }
    }
    
    // 写入文件
    if fs::write(&path, &input.content).is_err() {
        return WriteStateOutput {
            success: false,
            content: None,
        };
    }
    
    // 复用 read_state_json 返回最新内容
    let final_read = read_state_json(ReadStateInput { path });
    WriteStateOutput {
        success: true,
        content: final_read.content,
    }
}

/// 删除状态文件
/// 文件不存在视为成功，但 deleted = false
pub fn delete_state_json(input: DeleteStateInput) -> DeleteStateOutput {
    let path = input.path;
    
    // 文件不存在，不触发删除
    if !path.exists() {
        return DeleteStateOutput {
            deleted: false,
            path,
        };
    }
    
    // 删除文件
    match fs::remove_file(&path) {
        Ok(_) => DeleteStateOutput {
            deleted: true,
            path,
        },
        Err(_) => DeleteStateOutput {
            deleted: false,
            path,
        },
    }
}
