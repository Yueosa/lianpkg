//! TEX 文件二进制读取器（内部使用）

use std::io::{Read, Seek};
use byteorder::{ReadBytesExt, LittleEndian};
use crate::core::error::{CoreError, CoreResult};
use crate::core::tex::structs::*;

/// 读取 TEX 文件结构
pub(crate) fn read_tex<R: Read + Seek>(mut reader: R) -> CoreResult<TexFile> {
    let magic1 = read_null_terminated_string(&mut reader, 16)?;
    if magic1 != "TEXV0005" {
        return Err(CoreError::invalid_data(format!("Invalid Magic1: '{}'", magic1)));
    }

    let magic2 = read_null_terminated_string(&mut reader, 16)?;
    if magic2 != "TEXI0001" {
        return Err(CoreError::invalid_data(format!("Invalid Magic2: '{}'", magic2)));
    }

    let header = read_header(&mut reader)?;
    let images = read_image_container(&mut reader, &header)?;

    Ok(TexFile {
        header,
        images,
    })
}

fn read_header<R: Read + Seek>(reader: &mut R) -> CoreResult<TexHeader> {
    let format = reader.read_u32::<LittleEndian>().map_err(read_err)?;
    let flags = reader.read_u32::<LittleEndian>().map_err(read_err)?;
    let texture_width = reader.read_u32::<LittleEndian>().map_err(read_err)?;
    let texture_height = reader.read_u32::<LittleEndian>().map_err(read_err)?;
    let image_width = reader.read_u32::<LittleEndian>().map_err(read_err)?;
    let image_height = reader.read_u32::<LittleEndian>().map_err(read_err)?;
    let unk_int0 = reader.read_u32::<LittleEndian>().map_err(read_err)?;

    Ok(TexHeader {
        format,
        flags,
        texture_width,
        texture_height,
        image_width,
        image_height,
        unk_int0,
    })
}

fn read_image_container<R: Read + Seek>(reader: &mut R, _header: &TexHeader) -> CoreResult<Vec<TexImage>> {
    let magic = read_fixed_string(reader, 16)?;
    let image_count = reader.read_i32::<LittleEndian>().map_err(read_err)?;

    // 合理性检查
    if image_count < 0 || image_count > 1000 {
        return Err(CoreError::invalid_data(format!(
            "image_count {} out of valid range [0, 1000]", image_count
        )));
    }

    let mut image_format: i32 = -1; // Default to FIF_UNKNOWN
    let mut is_video_mp4 = false;
    let mut version = 0;

    if let Some(stripped) = magic.strip_prefix("TEXB") {
        if let Ok(v) = stripped.parse::<i32>() {
            version = v;
        }
    }

    match magic.as_str() {
        "TEXB0001" | "TEXB0002" => {},
        "TEXB0003" => {
            image_format = reader.read_i32::<LittleEndian>().map_err(read_err)?;
        },
        "TEXB0004" => {
            image_format = reader.read_i32::<LittleEndian>().map_err(read_err)?;
            is_video_mp4 = reader.read_i32::<LittleEndian>().map_err(read_err)? == 1;
        },
        _ => return Err(CoreError::invalid_data(format!("Unknown ImageContainer Magic: '{}'", magic))),
    }

    let effective_version = if version == 4 && !is_video_mp4 { 3 } else { version };

    let mut images = Vec::new();
    for _ in 0..image_count {
        images.push(read_image(reader, effective_version, image_format, is_video_mp4)?);
    }

    Ok(images)
}

fn read_image<R: Read + Seek>(reader: &mut R, version: i32, image_format: i32, is_video_mp4: bool) -> CoreResult<TexImage> {
    let mipmap_count = reader.read_i32::<LittleEndian>().map_err(read_err)?;

    // 合理性检查
    if mipmap_count < 0 || mipmap_count > 100 {
        return Err(CoreError::invalid_data(format!(
            "mipmap_count {} out of valid range [0, 100]", mipmap_count
        )));
    }

    let mut mipmaps = Vec::new();

    for _ in 0..mipmap_count {
        mipmaps.push(read_mipmap(reader, version)?);
    }

    Ok(TexImage {
        image_format,
        is_video_mp4,
        mipmaps,
    })
}

fn read_mipmap<R: Read + Seek>(reader: &mut R, version: i32) -> CoreResult<TexMipmap> {
    if version == 4 {
        // V4 specific fields
        let _param1 = reader.read_i32::<LittleEndian>().map_err(read_err)?;
        let _param2 = reader.read_i32::<LittleEndian>().map_err(read_err)?;
        let _condition_json = read_null_terminated_string(reader, 4096)?;
        let _param3 = reader.read_i32::<LittleEndian>().map_err(read_err)?;
    }

    let width = reader.read_u32::<LittleEndian>().map_err(read_err)?;
    let height = reader.read_u32::<LittleEndian>().map_err(read_err)?;

    let mut is_lz4_compressed = false;
    let mut decompressed_bytes_count = 0;

    if version >= 2 {
        is_lz4_compressed = reader.read_i32::<LittleEndian>().map_err(read_err)? == 1;
        decompressed_bytes_count = reader.read_u32::<LittleEndian>().map_err(read_err)?;
    }

    let byte_count = reader.read_i32::<LittleEndian>().map_err(read_err)?;

    // 合理性检查
    if byte_count < 0 {
        return Err(CoreError::invalid_data(format!(
            "mipmap byte_count {} is negative", byte_count
        )));
    }
    // 防止恶意文件触发巨量分配（512 MB 上限）
    if byte_count as u64 > 512 * 1024 * 1024 {
        return Err(CoreError::invalid_data(format!(
            "mipmap byte_count {} exceeds 512 MB limit", byte_count
        )));
    }

    let mut data = vec![0u8; byte_count as usize];
    reader.read_exact(&mut data).map_err(read_err)?;

    Ok(TexMipmap {
        width,
        height,
        is_lz4_compressed,
        decompressed_bytes_count,
        data,
    })
}

/// 读取固定长度字段：精确消费 `field_size` 字节，在已读字节中找第一个 `\0` 截取字符串。
///
/// 消除旧 `read_n_string` 不消费 padding 导致偏移错位的问题。
fn read_fixed_string<R: Read>(reader: &mut R, field_size: usize) -> CoreResult<String> {
    let mut buf = vec![0u8; field_size];
    reader.read_exact(&mut buf).map_err(read_err)?;

    // 找第一个 \0，截取前面部分
    let end = buf.iter().position(|&b| b == 0).unwrap_or(field_size);
    String::from_utf8(buf[..end].to_vec()).map_err(|e| {
        CoreError::invalid_data(format!("read_fixed_string: invalid UTF-8: {}", e))
    })
}

/// 读取 null-terminated 字符串（变长），带安全上限。
///
/// 用于 V4 mipmap 的 condition_json 等变长字段。
fn read_null_terminated_string<R: Read>(reader: &mut R, max_bytes: usize) -> CoreResult<String> {
    let mut bytes = Vec::new();
    let mut c = [0u8; 1];

    loop {
        reader.read_exact(&mut c).map_err(read_err)?;
        if c[0] == 0 {
            break;
        }
        bytes.push(c[0]);
        if bytes.len() >= max_bytes {
            return Err(CoreError::invalid_data(format!(
                "null-terminated string exceeds {} byte limit", max_bytes
            )));
        }
    }

    String::from_utf8(bytes).map_err(|e| {
        CoreError::invalid_data(format!("read_null_terminated_string: invalid UTF-8: {}", e))
    })
}

/// 将 `io::Error` 转换为 `CoreError::InvalidData`
fn read_err(e: std::io::Error) -> CoreError {
    CoreError::invalid_data(format!("TEX read error: {}", e))
}
