use std::path::Path;
use std::fs::File;
use std::io::{self, Write};
use super::structs::*;
use image::RgbaImage;
use texture2ddecoder::{decode_bc1, decode_bc2, decode_bc3};

pub fn convert_and_save(tex_file: &TexFile, output_path: &Path) -> io::Result<()> {
    if let Some(first_image) = tex_file.images.first() {
        if let Some(first_mipmap) = first_image.mipmaps.first() {
            let format = determine_format(tex_file, first_image);
            
            let mut data = first_mipmap.data.clone();

            // 1. Decompress LZ4 if needed
            if first_mipmap.is_lz4_compressed {
                data = lz4_flex::decompress(&data, first_mipmap.decompressed_bytes_count as usize)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("LZ4 decompression failed: {}", e)))?;
            }

            // 2. Handle MP4
            if format == MipmapFormat::VideoMp4 {
                let mut path = output_path.to_path_buf();
                path.set_extension("mp4");
                let mut file = File::create(path)?;
                file.write_all(&data)?;
                return Ok(());
            }

            // 3. Handle Images (already encoded files like PNG, JPEG)
            if format.is_image() {
                let ext = get_extension(format);
                let mut path = output_path.to_path_buf();
                path.set_extension(ext);
                let mut file = File::create(path)?;
                file.write_all(&data)?;
                return Ok(());
            }

            // 4. Handle Raw/Compressed Formats (DXT, RGBA, etc.)
            let width = first_mipmap.width as usize;
            let height = first_mipmap.height as usize;
            let decoded_data: Vec<u8>;

            match format {
                MipmapFormat::CompressedDXT1 => {
                    let mut pixels = vec![0u32; width * height];
                    decode_bc1(&data, width, height, &mut pixels)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("DXT1 decode failed: {}", e)))?;
                    decoded_data = pixels.iter().flat_map(|&p| p.to_le_bytes()).collect();
                },
                MipmapFormat::CompressedDXT3 => {
                    let mut pixels = vec![0u32; width * height];
                    decode_bc2(&data, width, height, &mut pixels)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("DXT3 decode failed: {}", e)))?;
                    decoded_data = pixels.iter().flat_map(|&p| p.to_le_bytes()).collect();
                },
                MipmapFormat::CompressedDXT5 => {
                    let mut pixels = vec![0u32; width * height];
                    decode_bc3(&data, width, height, &mut pixels)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("DXT5 decode failed: {}", e)))?;
                    decoded_data = pixels.iter().flat_map(|&p| p.to_le_bytes()).collect();
                },
                MipmapFormat::RGBA8888 => {
                    decoded_data = data;
                },
                MipmapFormat::RG88 => {
                    // Convert RG88 to RGBA8888
                    let mut new_data = Vec::with_capacity(width * height * 4);
                    for chunk in data.chunks(2) {
                        new_data.push(chunk[0]); // R
                        new_data.push(chunk[1]); // G
                        new_data.push(0);        // B
                        new_data.push(255);      // A
                    }
                    decoded_data = new_data;
                },
                MipmapFormat::R8 => {
                    // Convert R8 to RGBA8888 (Grayscale)
                    let mut new_data = Vec::with_capacity(width * height * 4);
                    for &b in &data {
                        new_data.push(b); // R
                        new_data.push(b); // G
                        new_data.push(b); // B
                        new_data.push(255); // A
                    }
                    decoded_data = new_data;
                },
                _ => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unsupported format: {:?}", format)));
                }
            }

            // Save as PNG
            let mut path = output_path.to_path_buf();
            path.set_extension("png");
            
            if let Some(img) = RgbaImage::from_raw(width as u32, height as u32, decoded_data) {
                img.save(path).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to save image: {}", e)))?;
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to create image buffer"));
            }
        }
    }
    Ok(())
}

fn determine_format(tex_file: &TexFile, image: &TexImage) -> MipmapFormat {
    if image.is_video_mp4 {
        return MipmapFormat::VideoMp4;
    }

    // Check flags for IsVideoTexture (bit 5 = 32)
    if (tex_file.header.flags & 32) != 0 {
        return MipmapFormat::VideoMp4;
    }

    // If image_format is valid (>= 0), convert FreeImageFormat to MipmapFormat
    if image.image_format >= 0 {
        return free_image_format_to_mipmap_format(image.image_format);
    }

    // Otherwise use header format
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

fn free_image_format_to_mipmap_format(fif: i32) -> MipmapFormat {
    match fif {
        0 => MipmapFormat::ImageBMP,
        1 => MipmapFormat::ImageICO,
        2 => MipmapFormat::ImageJPEG,
        3 => MipmapFormat::ImageJNG,
        4 => MipmapFormat::ImageKOALA,
        5 => MipmapFormat::ImageLBM, // IFF/LBM
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

fn get_extension(format: MipmapFormat) -> &'static str {
    match format {
        MipmapFormat::ImageBMP => "bmp",
        MipmapFormat::ImageICO => "ico",
        MipmapFormat::ImageJPEG => "jpg",
        MipmapFormat::ImageJNG => "jng",
        MipmapFormat::ImageKOALA => "koa",
        MipmapFormat::ImageLBM => "iff",
        MipmapFormat::ImageMNG => "mng",
        MipmapFormat::ImagePBM | MipmapFormat::ImagePBMRAW => "pbm",
        MipmapFormat::ImagePCD => "pcd",
        MipmapFormat::ImagePCX => "pcx",
        MipmapFormat::ImagePGM | MipmapFormat::ImagePGMRAW => "pgm",
        MipmapFormat::ImagePNG => "png",
        MipmapFormat::ImagePPM | MipmapFormat::ImagePPMRAW => "ppm",
        MipmapFormat::ImageRAS => "ras",
        MipmapFormat::ImageTARGA => "tga",
        MipmapFormat::ImageTIFF => "tif",
        MipmapFormat::ImageWBMP => "wbmp",
        MipmapFormat::ImagePSD => "psd",
        MipmapFormat::ImageCUT => "cut",
        MipmapFormat::ImageXBM => "xbm",
        MipmapFormat::ImageXPM => "xpm",
        MipmapFormat::ImageDDS => "dds",
        MipmapFormat::ImageGIF => "gif",
        MipmapFormat::ImageHDR => "hdr",
        MipmapFormat::ImageFAXG3 => "g3",
        MipmapFormat::ImageSGI => "sgi",
        MipmapFormat::ImageEXR => "exr",
        MipmapFormat::ImageJ2K => "j2k",
        MipmapFormat::ImageJP2 => "jp2",
        MipmapFormat::ImagePFM => "pfm",
        MipmapFormat::ImagePICT => "pict",
        MipmapFormat::ImageRAW => "raw",
        MipmapFormat::VideoMp4 => "mp4",
        _ => "bin",
    }
}
