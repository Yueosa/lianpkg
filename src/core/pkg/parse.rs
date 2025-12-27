//! 解析接口 - 只读取元数据，不写入文件

use std::fs;

use crate::core::pkg::structs::{
    ParsePkgInput, ParsePkgOutput,
    PkgInfo, PkgEntry,
};
use crate::core::pkg::utl::Reader;

/// 解析 pkg 文件，返回元数据信息
/// 只读取不写入，用于预览或决定是否解包
pub fn parse_pkg(input: ParsePkgInput) -> ParsePkgOutput {
    let file_path = input.file_path;

    // 读取文件
    let data = match fs::read(&file_path) {
        Ok(d) => d,
        Err(e) => {
            return ParsePkgOutput {
                success: false,
                pkg_info: None,
                error: Some(format!("Failed to read file {:?}: {}", file_path, e)),
            };
        }
    };

    // 解析文件
    parse_pkg_data(&data)
}

/// 从字节数据解析 pkg 信息（内部函数，供 unpack 复用）
pub(crate) fn parse_pkg_data(data: &[u8]) -> ParsePkgOutput {
    let mut r = Reader::new(data);

    // 读取版本
    let version = r.read_string();
    
    // 读取文件数量
    let file_count = r.read_u32();

    // 读取文件条目
    let mut entries = Vec::with_capacity(file_count as usize);
    for _ in 0..file_count {
        let name = r.read_string();
        let offset = r.read_u32();
        let size = r.read_u32();
        entries.push(PkgEntry { name, offset, size });
    }

    // 记录数据区起始位置
    let data_start = r.position();

    ParsePkgOutput {
        success: true,
        pkg_info: Some(PkgInfo {
            version,
            file_count,
            entries,
            data_start,
        }),
        error: None,
    }
}
