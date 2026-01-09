//! 转换接口 - 解析并转换 TEX 文件

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use image::RgbaImage;

use crate::core::error::{CoreError, CoreResult};
use crate::core::tex::decoder::{decode_mipmap, determine_format};
use crate::core::tex::reader;
use crate::core::tex::structs::{
    ConvertTexInput, ConvertTexOutput, ConvertedFile, MipmapFormat, TexInfo,
};

/// 解析并转换 TEX 文件
pub fn convert_tex(input: ConvertTexInput) -> CoreResult<ConvertTexOutput> {
    let file_path = input.file_path;
    let output_path = input.output_path;

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

    // 获取第一个图像和 mipmap
    let first_image = tex_file
        .images
        .first()
        .ok_or_else(|| CoreError::Validation {
            message: "No images found in TEX file".to_string(),
        })?;

    let first_mipmap = first_image
        .mipmaps
        .first()
        .ok_or_else(|| CoreError::Validation {
            message: "No mipmaps found in TEX image".to_string(),
        })?;

    // 确定格式
    let format = determine_format(&tex_file, first_image);
    let width = first_mipmap.width;
    let height = first_mipmap.height;

    // 构建 TexInfo
    let tex_info = TexInfo {
        version: "TEXV0005".to_string(),
        format: format.name().to_string(),
        width,
        height,
        image_count: tex_file.images.len(),
        mipmap_count: first_image.mipmaps.len(),
        is_compressed: first_mipmap.is_lz4_compressed,
        is_video: first_image.is_video_mp4 || (tex_file.header.flags & 32) != 0,
        data_size: first_mipmap.data.len(),
    };

    // 解压 LZ4（如果需要）
    let data = if first_mipmap.is_lz4_compressed {
        lz4_flex::decompress(
            &first_mipmap.data,
            first_mipmap.decompressed_bytes_count as usize,
        )
        .map_err(|e| CoreError::Parse {
            message: format!("LZ4 decompression failed: {}", e),
            source: Some(file_path.display().to_string()),
        })?
    } else {
        first_mipmap.data.clone()
    };

    // 确定输出路径
    let mut final_output_path = output_path.clone();
    let ext = format.extension();

    // 如果输出路径是目录，使用输入文件名
    if output_path.is_dir() || !output_path.to_string_lossy().contains('.') {
        if output_path.is_dir() {
            let stem = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            final_output_path = output_path.join(format!("{}.{}", stem, ext));
        } else {
            final_output_path.set_extension(ext);
        }
    } else {
        final_output_path.set_extension(ext);
    }

    // 确保输出目录存在
    if let Some(parent) = final_output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CoreError::Io {
            message: e.to_string(),
            path: Some(parent.display().to_string()),
        })?;
    }

    // 处理不同格式
    let result = match format {
        MipmapFormat::VideoMp4 => save_raw_data(&final_output_path, &data),
        f if f.is_image() => save_raw_data(&final_output_path, &data),
        _ => match decode_mipmap(&data, width as usize, height as usize, format) {
            Ok(decoded) => save_as_png(&final_output_path, &decoded, width, height),
            Err(e) => Err(e),
        },
    };

    result.map_err(|e| CoreError::Io {
        message: e,
        path: Some(final_output_path.display().to_string()),
    })?;

    Ok(ConvertTexOutput {
        converted_file: ConvertedFile {
            output_path: final_output_path,
            format: ext.to_string(),
            width,
            height,
        },
        tex_info,
    })
}

/// 保存原始数据到文件
fn save_raw_data(path: &PathBuf, data: &[u8]) -> Result<(), String> {
    let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
    file.write_all(data)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

/// 保存为 PNG 图片
fn save_as_png(path: &PathBuf, data: &[u8], width: u32, height: u32) -> Result<(), String> {
    let img = RgbaImage::from_raw(width, height, data.to_vec())
        .ok_or_else(|| "Failed to create image buffer".to_string())?;

    img.save(path)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(())
}
