//! 解析接口 - 读取 TEX 文件元数据

use std::fs::File;

use crate::core::error::{CoreError, CoreResult};
use crate::core::tex::decoder::determine_format;
use crate::core::tex::reader;
use crate::core::tex::structs::{ParseTexInput, ParseTexOutput, TexInfo};

/// 解析 TEX 文件，只读取元数据不进行转换
pub fn parse_tex(input: ParseTexInput) -> CoreResult<ParseTexOutput> {
    let file_path = input.file_path;

    // 打开文件
    let mut file = File::open(&file_path).map_err(|e| CoreError::Io {
        message: e.to_string(),
        path: Some(file_path.display().to_string()),
    })?;

    // 读取 TEX 结构
    let tex_file = reader::read_tex(&mut file).map_err(|e| CoreError::Parse {
        message: e.to_string(),
        source: Some(file_path.display().to_string()),
    })?;

    // 提取信息
    let first_image = tex_file.images.first();
    let first_mipmap = first_image.and_then(|img| img.mipmaps.first());

    let format = first_image
        .map(|img| determine_format(&tex_file, img))
        .unwrap_or(crate::core::tex::structs::MipmapFormat::Invalid);

    let (width, height) = first_mipmap.map(|m| (m.width, m.height)).unwrap_or((0, 0));

    let is_compressed = first_mipmap.map(|m| m.is_lz4_compressed).unwrap_or(false);

    let is_video = first_image
        .map(|img| img.is_video_mp4 || (tex_file.header.flags & 32) != 0)
        .unwrap_or(false);

    let mipmap_count = first_image.map(|img| img.mipmaps.len()).unwrap_or(0);

    let data_size = first_mipmap.map(|m| m.data.len()).unwrap_or(0);

    let tex_info = TexInfo {
        version: "TEXV0005".to_string(),
        format: format.name().to_string(),
        width,
        height,
        image_count: tex_file.images.len(),
        mipmap_count,
        is_compressed,
        is_video,
        data_size,
    };

    Ok(ParseTexOutput { tex_info })
}
