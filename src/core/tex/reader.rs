//! TEX 文件二进制读取器（内部使用）

use std::io::{self, Read, Seek};
use byteorder::{ReadBytesExt, LittleEndian};
use crate::core::tex::structs::*;

/// 读取 TEX 文件结构
pub(crate) fn read_tex<R: Read + Seek>(mut reader: R) -> io::Result<TexFile> {
    let magic1 = read_n_string(&mut reader, 16)?;
    if magic1 != "TEXV0005" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid Magic1: {}", magic1)));
    }

    let magic2 = read_n_string(&mut reader, 16)?;
    if magic2 != "TEXI0001" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid Magic2: {}", magic2)));
    }

    let header = read_header(&mut reader)?;
    let images = read_image_container(&mut reader, &header)?;

    Ok(TexFile {
        header,
        images,
    })
}

fn read_header<R: Read + Seek>(reader: &mut R) -> io::Result<TexHeader> {
    let format = reader.read_u32::<LittleEndian>()?;
    let flags = reader.read_u32::<LittleEndian>()?;
    let texture_width = reader.read_u32::<LittleEndian>()?;
    let texture_height = reader.read_u32::<LittleEndian>()?;
    let image_width = reader.read_u32::<LittleEndian>()?;
    let image_height = reader.read_u32::<LittleEndian>()?;
    let unk_int0 = reader.read_u32::<LittleEndian>()?;

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

fn read_image_container<R: Read + Seek>(reader: &mut R, _header: &TexHeader) -> io::Result<Vec<TexImage>> {
    let magic = read_n_string(reader, 16)?;
    let image_count = reader.read_i32::<LittleEndian>()?;

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
            image_format = reader.read_i32::<LittleEndian>()?;
        },
        "TEXB0004" => {
            image_format = reader.read_i32::<LittleEndian>()?;
            is_video_mp4 = reader.read_i32::<LittleEndian>()? == 1;
        },
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unknown ImageContainer Magic: {}", magic))),
    }

    let effective_version = if version == 4 && !is_video_mp4 { 3 } else { version };

    let mut images = Vec::new();
    for _ in 0..image_count {
        images.push(read_image(reader, effective_version, image_format, is_video_mp4)?);
    }

    Ok(images)
}

fn read_image<R: Read + Seek>(reader: &mut R, version: i32, image_format: i32, is_video_mp4: bool) -> io::Result<TexImage> {
    let mipmap_count = reader.read_i32::<LittleEndian>()?;
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

fn read_mipmap<R: Read + Seek>(reader: &mut R, version: i32) -> io::Result<TexMipmap> {
    if version == 4 {
        // V4 specific fields
        let _param1 = reader.read_i32::<LittleEndian>()?;
        let _param2 = reader.read_i32::<LittleEndian>()?;
        let _condition_json = read_n_string(reader, 0)?;
        let _param3 = reader.read_i32::<LittleEndian>()?;
    }

    // Common fields for V2, V3, V4 (after V4 header)
    // V1 is different.

    let width = reader.read_u32::<LittleEndian>()?;
    let height = reader.read_u32::<LittleEndian>()?;

    let mut is_lz4_compressed = false;
    let mut decompressed_bytes_count = 0;

    if version >= 2 {
        is_lz4_compressed = reader.read_i32::<LittleEndian>()? == 1;
        decompressed_bytes_count = reader.read_u32::<LittleEndian>()?;
    }

    let byte_count = reader.read_i32::<LittleEndian>()?;
    let mut data = vec![0u8; byte_count as usize];
    reader.read_exact(&mut data)?;

    Ok(TexMipmap {
        width,
        height,
        is_lz4_compressed,
        decompressed_bytes_count,
        data,
    })
}

fn read_n_string<R: Read + Seek>(reader: &mut R, max_length: usize) -> io::Result<String> {
    let mut bytes = Vec::new();
    let mut c = [0u8; 1];
    
    loop {
        reader.read_exact(&mut c)?;
        if c[0] == 0 {
            break;
        }
        bytes.push(c[0]);
        if max_length > 0 && bytes.len() >= max_length {
            break;
        }
    }

    Ok(String::from_utf8_lossy(&bytes).to_string())
}
