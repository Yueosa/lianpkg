//! 解析接口 - 只读取元数据，不写入文件

use std::fs;

use crate::core::error::{CoreError, CoreResult};
use crate::core::pkg::structs::{ParsePkgInput, ParsePkgOutput, PkgEntry, PkgInfo};
use crate::core::pkg::utl::Reader;

/// 解析 pkg 文件，返回元数据信息
/// 只读取不写入，用于预览或决定是否解包
pub fn parse_pkg(input: ParsePkgInput) -> CoreResult<ParsePkgOutput> {
    let file_path = input.file_path;

    // 读取文件
    let data = fs::read(&file_path).map_err(|e| CoreError::Io {
        message: e.to_string(),
        path: Some(file_path.display().to_string()),
    })?;

    // 解析文件
    parse_pkg_data(&data)
}

/// 从字节数据解析 pkg 信息（内部函数，供 unpack 复用）
pub(crate) fn parse_pkg_data(data: &[u8]) -> CoreResult<ParsePkgOutput> {
    let mut r = Reader::new(data);

    // 读取版本
    let version = r.read_string()?;

    // 读取文件数量
    let file_count = r.read_u32()?;

    // 合理性检查：防止恶意/损坏文件导致巨量内存分配
    if file_count > 100_000 {
        return Err(CoreError::invalid_data(format!(
            "file_count {} exceeds limit 100000",
            file_count
        )));
    }

    // 读取文件条目
    let mut entries = Vec::with_capacity(file_count as usize);
    for _ in 0..file_count {
        let name = r.read_string()?;
        let offset = r.read_u32()?;
        let size = r.read_u32()?;
        entries.push(PkgEntry { name, offset, size });
    }

    // 记录数据区起始位置
    let data_start = r.position();

    // 验证所有 entry 的 offset+size 不超出 data 范围
    for entry in &entries {
        let abs_end = data_start + entry.offset as usize + entry.size as usize;
        if abs_end > data.len() {
            return Err(CoreError::invalid_data(format!(
                "entry '{}': data_start({}) + offset({}) + size({}) = {} exceeds data length({})",
                entry.name, data_start, entry.offset, entry.size, abs_end, data.len()
            )));
        }
    }

    Ok(ParsePkgOutput {
        pkg_info: PkgInfo {
            version,
            file_count,
            entries,
            data_start,
        },
    })
}
