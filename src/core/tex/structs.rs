//! 结构体定义 - Input/Output、运行时结构体

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Input 结构体
// ============================================================================

/// parse_tex 接口入参
#[derive(Debug, Clone)]
pub struct ParseTexInput {
    /// TEX 文件路径
    pub file_path: PathBuf,
}

/// convert_tex 接口入参
#[derive(Debug, Clone)]
pub struct ConvertTexInput {
    /// TEX 文件路径
    pub file_path: PathBuf,
    /// 输出路径（目录或文件）
    pub output_path: PathBuf,
}

// ============================================================================
// Output 结构体
// ============================================================================

/// parse_tex 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseTexOutput {
    /// TEX 文件信息
    pub tex_info: TexInfo,
}

/// convert_tex 接口返回值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertTexOutput {
    /// 转换后的文件信息
    pub converted_file: ConvertedFile,
    /// TEX 文件信息
    pub tex_info: TexInfo,
}

// ============================================================================
// 运行时结构体（对外导出）
// ============================================================================

/// Tex 文件信息（解析结果，用于预览）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TexInfo {
    /// TEX 版本
    pub version: String,
    /// 格式类型
    pub format: String,
    /// 图像宽度
    pub width: u32,
    /// 图像高度
    pub height: u32,
    /// 图像数量
    pub image_count: usize,
    /// Mipmap 数量
    pub mipmap_count: usize,
    /// 是否 LZ4 压缩
    pub is_compressed: bool,
    /// 是否视频
    pub is_video: bool,
    /// 数据大小（字节）
    pub data_size: usize,
}

/// 转换后的文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedFile {
    /// 输出路径
    pub output_path: PathBuf,
    /// 输出格式 (png/mp4/jpg等)
    pub format: String,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
}

// ============================================================================
// 内部运行时结构体
// ============================================================================

/// TEX 文件完整结构（内部使用）
#[derive(Debug, Clone)]
pub struct TexFile {
    pub header: TexHeader,
    pub images: Vec<TexImage>,
}

/// TEX 文件头
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TexHeader {
    pub format: u32,
    pub flags: u32,
    pub texture_width: u32,
    pub texture_height: u32,
    pub image_width: u32,
    pub image_height: u32,
    pub unk_int0: u32,
}

/// TEX 图像
#[derive(Debug, Clone)]
pub struct TexImage {
    pub image_format: i32,
    pub is_video_mp4: bool,
    pub mipmaps: Vec<TexMipmap>,
}

/// TEX Mipmap
#[derive(Debug, Clone)]
pub struct TexMipmap {
    pub width: u32,
    pub height: u32,
    pub is_lz4_compressed: bool,
    pub decompressed_bytes_count: u32,
    pub data: Vec<u8>,
}

/// Mipmap 格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MipmapFormat {
    Invalid = 0,
    RGBA8888 = 1,
    R8 = 2,
    RG88 = 3,
    CompressedDXT5 = 4,
    CompressedDXT3 = 5,
    CompressedDXT1 = 6,
    VideoMp4 = 7,

    // 图片格式
    ImageBMP = 1000,
    ImageICO,
    ImageJPEG,
    ImageJNG,
    ImageKOALA,
    ImageLBM,
    ImageIFF,
    ImageMNG,
    ImagePBM,
    ImagePBMRAW,
    ImagePCD,
    ImagePCX,
    ImagePGM,
    ImagePGMRAW,
    ImagePNG,
    ImagePPM,
    ImagePPMRAW,
    ImageRAS,
    ImageTARGA,
    ImageTIFF,
    ImageWBMP,
    ImagePSD,
    ImageCUT,
    ImageXBM,
    ImageXPM,
    ImageDDS,
    ImageGIF,
    ImageHDR,
    ImageFAXG3,
    ImageSGI,
    ImageEXR,
    ImageJ2K,
    ImageJP2,
    ImagePFM,
    ImagePICT,
    ImageRAW,
}

impl MipmapFormat {
    /// 是否为图片格式
    pub fn is_image(&self) -> bool {
        *self as u32 >= 1000
    }

    /// 是否为压缩格式
    #[allow(dead_code)]
    pub fn is_compressed(&self) -> bool {
        matches!(
            self,
            MipmapFormat::CompressedDXT1
                | MipmapFormat::CompressedDXT3
                | MipmapFormat::CompressedDXT5
        )
    }

    /// 获取格式名称
    pub fn name(&self) -> &'static str {
        match self {
            MipmapFormat::Invalid => "Invalid",
            MipmapFormat::RGBA8888 => "RGBA8888",
            MipmapFormat::R8 => "R8",
            MipmapFormat::RG88 => "RG88",
            MipmapFormat::CompressedDXT5 => "DXT5",
            MipmapFormat::CompressedDXT3 => "DXT3",
            MipmapFormat::CompressedDXT1 => "DXT1",
            MipmapFormat::VideoMp4 => "MP4",
            MipmapFormat::ImagePNG => "PNG",
            MipmapFormat::ImageJPEG => "JPEG",
            MipmapFormat::ImageBMP => "BMP",
            MipmapFormat::ImageGIF => "GIF",
            _ => "Image",
        }
    }

    /// 获取文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            MipmapFormat::ImageBMP => "bmp",
            MipmapFormat::ImageICO => "ico",
            MipmapFormat::ImageJPEG => "jpg",
            MipmapFormat::ImageJNG => "jng",
            MipmapFormat::ImageKOALA => "koa",
            MipmapFormat::ImageLBM | MipmapFormat::ImageIFF => "iff",
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
            _ => "png",
        }
    }
}
