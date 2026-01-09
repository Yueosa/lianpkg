//! 解包接口 - 解析并解包 pkg 文件

use std::fs;

use crate::core::error::{CoreError, CoreResult};
use crate::core::pkg::parse::parse_pkg_data;
use crate::core::pkg::structs::{
    ExtractedFile, UnpackEntryInput, UnpackEntryOutput, UnpackPkgInput, UnpackPkgOutput,
};

/// 解包整个 pkg 文件
/// 解析元数据并提取所有文件到输出目录
pub fn unpack_pkg(input: UnpackPkgInput) -> CoreResult<UnpackPkgOutput> {
    let file_path = input.file_path;
    let output_base = input.output_base;

    // 读取文件
    let data = fs::read(&file_path).map_err(|e| CoreError::Io {
        message: e.to_string(),
        path: Some(file_path.display().to_string()),
    })?;

    // 解析 pkg
    let parse_result = parse_pkg_data(&data)?;
    let pkg_info = parse_result.pkg_info;
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
        })?;

        extracted_files.push(ExtractedFile {
            entry_name: entry.name.clone(),
            output_path: result.output_path,
            size: entry.size,
        });
    }

    Ok(UnpackPkgOutput {
        pkg_info,
        extracted_files,
    })
}

/// 解包单个条目
/// 用于精细控制，选择性解包特定文件
pub fn unpack_entry(input: UnpackEntryInput) -> CoreResult<UnpackEntryOutput> {
    let data = &input.pkg_data;
    let data_start = input.data_start;
    let entry = &input.entry;
    let output_path = input.output_path;

    // 计算数据位置
    let start = data_start + entry.offset as usize;
    let end = start + entry.size as usize;

    // 边界检查
    if end > data.len() {
        return Err(CoreError::Validation {
            message: format!("Entry {} out of bounds", entry.name),
        });
    }

    // 提取内容
    let content = &data[start..end];

    // 确保父目录存在
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| CoreError::Io {
            message: e.to_string(),
            path: Some(parent.display().to_string()),
        })?;
    }

    // 写入文件
    fs::write(&output_path, content).map_err(|e| CoreError::Io {
        message: e.to_string(),
        path: Some(output_path.display().to_string()),
    })?;

    Ok(UnpackEntryOutput { output_path })
}
