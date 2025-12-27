//! 格式解码器（内部使用）

use texture2ddecoder::{decode_bc1, decode_bc2, decode_bc3};
use crate::core::tex::structs::{TexFile, TexImage, MipmapFormat};

/// 确定 Mipmap 格式
pub(crate) fn determine_format(tex_file: &TexFile, image: &TexImage) -> MipmapFormat {
    // 检查是否为视频
    if image.is_video_mp4 {
        return MipmapFormat::VideoMp4;
    }

    // 检查 flags 中的 IsVideoTexture 位 (bit 5 = 32)
    if (tex_file.header.flags & 32) != 0 {
        return MipmapFormat::VideoMp4;
    }

    // 如果 image_format 有效 (>= 0)，转换 FreeImageFormat 到 MipmapFormat
    if image.image_format >= 0 {
        return free_image_format_to_mipmap_format(image.image_format);
    }

    // 否则使用 header format
    match tex_file.header.format {
        0 => MipmapFormat::RGBA8888,
        4 => MipmapFormat::CompressedDXT5,
        6 => MipmapFormat::CompressedDXT3,
        7 => MipmapFormat::CompressedDXT1,
        8 => MipmapFormat::RG88,
        9 => MipmapFormat::R8,
        _ => MipmapFormat::Invalid,
    }
}

/// FreeImage 格式转换为 MipmapFormat
fn free_image_format_to_mipmap_format(fif: i32) -> MipmapFormat {
    match fif {
        0 => MipmapFormat::ImageBMP,
        1 => MipmapFormat::ImageICO,
        2 => MipmapFormat::ImageJPEG,
        3 => MipmapFormat::ImageJNG,
        4 => MipmapFormat::ImageKOALA,
        5 => MipmapFormat::ImageLBM,
        6 => MipmapFormat::ImageMNG,
        7 => MipmapFormat::ImagePBM,
        8 => MipmapFormat::ImagePBMRAW,
        9 => MipmapFormat::ImagePCD,
        10 => MipmapFormat::ImagePCX,
        11 => MipmapFormat::ImagePGM,
        12 => MipmapFormat::ImagePGMRAW,
        13 => MipmapFormat::ImagePNG,
        14 => MipmapFormat::ImagePPM,
        15 => MipmapFormat::ImagePPMRAW,
        16 => MipmapFormat::ImageRAS,
        17 => MipmapFormat::ImageTARGA,
        18 => MipmapFormat::ImageTIFF,
        19 => MipmapFormat::ImageWBMP,
        20 => MipmapFormat::ImagePSD,
        21 => MipmapFormat::ImageCUT,
        22 => MipmapFormat::ImageXBM,
        23 => MipmapFormat::ImageXPM,
        24 => MipmapFormat::ImageDDS,
        25 => MipmapFormat::ImageGIF,
        26 => MipmapFormat::ImageHDR,
        27 => MipmapFormat::ImageFAXG3,
        28 => MipmapFormat::ImageSGI,
        29 => MipmapFormat::ImageEXR,
        30 => MipmapFormat::ImageJ2K,
        31 => MipmapFormat::ImageJP2,
        32 => MipmapFormat::ImagePFM,
        33 => MipmapFormat::ImagePICT,
        34 => MipmapFormat::ImageRAW,
        35 => MipmapFormat::VideoMp4,
        _ => MipmapFormat::Invalid,
    }
}

/// 解码 Mipmap 数据为 RGBA
pub(crate) fn decode_mipmap(data: &[u8], width: usize, height: usize, format: MipmapFormat) -> Result<Vec<u8>, String> {
    match format {
        MipmapFormat::CompressedDXT1 => {
            let mut pixels = vec![0u32; width * height];
            decode_bc1(data, width, height, &mut pixels)
                .map_err(|e| format!("DXT1 decode failed: {}", e))?;
            Ok(pixels.iter().flat_map(|&p| p.to_le_bytes()).collect())
        }
        MipmapFormat::CompressedDXT3 => {
            let mut pixels = vec![0u32; width * height];
            decode_bc2(data, width, height, &mut pixels)
                .map_err(|e| format!("DXT3 decode failed: {}", e))?;
            Ok(pixels.iter().flat_map(|&p| p.to_le_bytes()).collect())
        }
        MipmapFormat::CompressedDXT5 => {
            let mut pixels = vec![0u32; width * height];
            decode_bc3(data, width, height, &mut pixels)
                .map_err(|e| format!("DXT5 decode failed: {}", e))?;
            Ok(pixels.iter().flat_map(|&p| p.to_le_bytes()).collect())
        }
        MipmapFormat::RGBA8888 => {
            Ok(data.to_vec())
        }
        MipmapFormat::RG88 => {
            // 转换 RG88 为 RGBA8888
            let mut new_data = Vec::with_capacity(width * height * 4);
            for chunk in data.chunks(2) {
                if chunk.len() >= 2 {
                    new_data.push(chunk[0]); // R
                    new_data.push(chunk[1]); // G
                    new_data.push(0);        // B
                    new_data.push(255);      // A
                }
            }
            Ok(new_data)
        }
        MipmapFormat::R8 => {
            // 转换 R8 为 RGBA8888 (灰度图)
            let mut new_data = Vec::with_capacity(width * height * 4);
            for &b in data {
                new_data.push(b);   // R
                new_data.push(b);   // G
                new_data.push(b);   // B
                new_data.push(255); // A
            }
            Ok(new_data)
        }
        _ => {
            Err(format!("Unsupported format for decoding: {:?}", format))
        }
    }
}
