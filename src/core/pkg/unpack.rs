//! 解包接口 - 解析并解包 pkg 文件

use std::fs;

use crate::core::pkg::structs::{
    UnpackPkgInput, UnpackPkgOutput,
    UnpackEntryInput, UnpackEntryOutput,
    ExtractedFile,
};
use crate::core::pkg::parse::parse_pkg_data;

/// 解包整个 pkg 文件
/// 解析元数据并提取所有文件到输出目录
pub fn unpack_pkg(input: UnpackPkgInput) -> UnpackPkgOutput {
    let file_path = input.file_path;
    let output_base = input.output_base;

    // 读取文件
    let data = match fs::read(&file_path) {
        Ok(d) => d,
        Err(e) => {
            return UnpackPkgOutput {
                success: false,
                pkg_info: None,
                extracted_files: Vec::new(),
                error: Some(format!("Failed to read file {:?}: {}", file_path, e)),
            };
        }
    };

    // 解析 pkg
    let parse_result = parse_pkg_data(&data);
    if !parse_result.success {
        return UnpackPkgOutput {
            success: false,
            pkg_info: None,
            extracted_files: Vec::new(),
            error: parse_result.error,
        };
    }

    let pkg_info = parse_result.pkg_info.unwrap();
    let data_start = pkg_info.data_start;
    let mut extracted_files = Vec::new();

    // 解包每个条目
    for entry in &pkg_info.entries {
        let output_path = output_base.join(&entry.name);
        
        let result = unpack_entry(UnpackEntryInput {
            pkg_data: data.clone(),
            data_start,
            entry: entry.clone(),
            output_path: output_path.clone(),
        });

        if result.success {
            extracted_files.push(ExtractedFile {
                entry_name: entry.name.clone(),
                output_path,
                size: entry.size,
            });
        } else {
            return UnpackPkgOutput {
                success: false,
                pkg_info: Some(pkg_info),
                extracted_files,
                error: result.error,
            };
        }
    }

    UnpackPkgOutput {
        success: true,
        pkg_info: Some(pkg_info),
        extracted_files,
        error: None,
    }
}

/// 解包单个条目
/// 用于精细控制，选择性解包特定文件
pub fn unpack_entry(input: UnpackEntryInput) -> UnpackEntryOutput {
    let data = &input.pkg_data;
    let data_start = input.data_start;
    let entry = &input.entry;
    let output_path = input.output_path;

    // 计算数据位置
    let start = data_start + entry.offset as usize;
    let end = start + entry.size as usize;

    // 边界检查
    if end > data.len() {
        return UnpackEntryOutput {
            success: false,
            output_path,
            error: Some(format!("Entry {} out of bounds", entry.name)),
        };
    }

    // 提取内容
    let content = &data[start..end];

    // 确保父目录存在
    if let Some(parent) = output_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            let err_msg = format!("Failed to create directory {:?}: {}", parent, e);
            return UnpackEntryOutput {
                success: false,
                output_path,
                error: Some(err_msg),
            };
        }
    }

    // 写入文件
    if let Err(e) = fs::write(&output_path, content) {
        let err_msg = format!("Failed to write file {:?}: {}", output_path, e);
        return UnpackEntryOutput {
            success: false,
            output_path,
            error: Some(err_msg),
        };
    }

    UnpackEntryOutput {
        success: true,
        output_path,
        error: None,
    }
}
