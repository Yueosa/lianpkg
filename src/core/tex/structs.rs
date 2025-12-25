
#[derive(Debug, Clone)]
pub struct TexFile {
    pub header: TexHeader,
    pub images: Vec<TexImage>,
}

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
    
    // Images
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
    pub fn is_image(&self) -> bool {
        *self as u32 >= 1000
    }

    #[allow(dead_code)]
    pub fn is_compressed(&self) -> bool {
        matches!(self, MipmapFormat::CompressedDXT1 | MipmapFormat::CompressedDXT3 | MipmapFormat::CompressedDXT5)
    }
}

#[derive(Debug, Clone)]
pub struct TexImage {
    pub image_format: i32,
    pub is_video_mp4: bool,
    pub mipmaps: Vec<TexMipmap>,
}

#[derive(Debug, Clone)]
pub struct TexMipmap {
    pub width: u32,
    pub height: u32,
    pub is_lz4_compressed: bool,
    pub decompressed_bytes_count: u32,
    pub data: Vec<u8>,
}

impl TexImage {
    #[allow(dead_code)]
    pub fn is_compressed(&self) -> bool {
        // Logic to determine if the format implies compression (like DXT)
        // This will be handled in the converter
        false
    }
}
