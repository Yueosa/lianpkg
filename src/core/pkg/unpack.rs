//! 解包接口 - 解析并解包 pkg 文件

use std::fs;
use std::path::Path;

use crate::core::error::{CoreError, CoreResult};
use crate::core::pkg::parse::parse_pkg_data;
use crate::core::pkg::structs::{
    ExtractedFile, PkgEntry, UnpackEntryInput, UnpackEntryOutput, UnpackPkgInput, UnpackPkgOutput,
};

/// 解包整个 pkg 文件
///
/// 解析元数据并提取所有文件到输出目录。
/// 内部使用零拷贝：整个 data 只读一次，entry 直接切片写出。
pub fn unpack_pkg(input: UnpackPkgInput) -> CoreResult<UnpackPkgOutput> {
    let file_path = input.file_path;
    let output_base = input.output_base;

    // 读取文件
    let data = fs::read(&file_path).map_err(|e| CoreError::Io {
        message: e.to_string(),
        path: Some(file_path.display().to_string()),
    })?;

    // 解析 pkg（含完整校验）
    let parse_result = parse_pkg_data(&data)?;
    let pkg_info = parse_result.pkg_info;
    let data_start = pkg_info.data_start;
    let mut extracted_files = Vec::new();

    // 解包每个条目（零拷贝：直接从 &data 切片写出）
    for entry in &pkg_info.entries {
        let output_path = output_base.join(&entry.name);
        write_entry(&data, data_start, entry, &output_path)?;

        extracted_files.push(ExtractedFile {
            entry_name: entry.name.clone(),
            output_path,
            size: entry.size,
        });
    }

    Ok(UnpackPkgOutput {
        pkg_info,
        extracted_files,
    })
}

/// 解包单个条目（公开接口，保持向后兼容）
///
/// 用于精细控制，选择性解包特定文件。
/// 内部委托给 `write_entry`，不再额外拷贝数据。
pub fn unpack_entry(input: UnpackEntryInput) -> CoreResult<UnpackEntryOutput> {
    let output_path = input.output_path;
    write_entry(&input.pkg_data, input.data_start, &input.entry, &output_path)?;
    Ok(UnpackEntryOutput { output_path })
}

/// 内部：将单个 entry 的数据切片写入文件
///
/// 边界检查已由 `parse_pkg_data` 完成，此处仅做防御性二次检查。
fn write_entry(
    data: &[u8],
    data_start: usize,
    entry: &PkgEntry,
    output_path: &Path,
) -> CoreResult<()> {
    let start = data_start + entry.offset as usize;
    let end = start + entry.size as usize;

    // 防御性边界检查
    if end > data.len() {
        return Err(CoreError::invalid_data(format!(
            "entry '{}': offset({}) + size({}) exceeds data length({})",
            entry.name, entry.offset, entry.size, data.len()
        )));
    }

    let content = &data[start..end];

    // 确保父目录存在
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| CoreError::Io {
            message: e.to_string(),
            path: Some(parent.display().to_string()),
        })?;
    }

    // 写入文件
    fs::write(output_path, content).map_err(|e| CoreError::Io {
        message: e.to_string(),
        path: Some(output_path.display().to_string()),
    })?;

    Ok(())
}
