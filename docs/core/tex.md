# tex 模块

## 概述

- **模块职责**: 解析和转换 Wallpaper Engine 的 TEX 纹理文件
- **使用场景**: 
  - 预览：`parse_tex` 查看 TEX 文件信息
  - 转换：`convert_tex` 转换为标准图片或视频格式
- **依赖关系**: 依赖 `error`、`path` 模块

## 接口列表

| 函数          | 输入              | 输出               | 说明                 |
| ------------- | ----------------- | ------------------ | -------------------- |
| `parse_tex`   | `ParseTexInput`   | `ParseTexOutput`   | 解析 TEX 文件头信息  |
| `convert_tex` | `ConvertTexInput` | `ConvertTexOutput` | 转换 TEX 为图片/视频 |

## 数据结构

### Input 结构体

#### ParseTexInput
```rust
pub struct ParseTexInput {
    /// TEX 文件路径
    pub file_path: PathBuf,
}
```

#### ConvertTexInput
```rust
pub struct ConvertTexInput {
    /// TEX 文件路径
    pub file_path: PathBuf,
    /// 输出路径（目录或文件）
    pub output_path: PathBuf,
}
```

### Output 结构体

#### ParseTexOutput
```rust
pub struct ParseTexOutput {
    /// TEX 文件信息
    pub tex_info: TexInfo,
}
```

#### ConvertTexOutput
```rust
pub struct ConvertTexOutput {
    /// 转换后的文件信息
    pub converted_file: ConvertedFile,
    /// TEX 文件信息
    pub tex_info: TexInfo,
}
```

### 运行时结构体

#### TexInfo
```rust
pub struct TexInfo {
    /// TEX 版本
    pub version: String,
    /// 格式类型（如 "DXT5", "RGBA8888", "PNG"）
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
```

#### ConvertedFile
```rust
pub struct ConvertedFile {
    /// 输出路径
    pub output_path: PathBuf,
    /// 输出格式 (png/mp4/jpg 等)
    pub format: String,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
}
```

#### MipmapFormat 枚举
```rust
pub enum MipmapFormat {
    // 原始格式
    Invalid = 0,
    RGBA8888 = 1,
    R8 = 2,
    RG88 = 3,
    
    // 压缩格式
    CompressedDXT5 = 4,
    CompressedDXT3 = 5,
    CompressedDXT1 = 6,
    
    // 视频格式
    VideoMp4 = 7,
    
    // 图片格式 (1000+)
    ImageBMP = 1000,
    ImageICO,
    ImageJPEG,
    ImagePNG,
    ImageGIF,
    // ... 更多图片格式
}
```

## 接口详解

### `parse_tex`

- **签名**: `fn parse_tex(input: ParseTexInput) -> CoreResult<ParseTexOutput>`
- **功能**: 解析 TEX 文件头，获取格式信息

**处理流程**:
1. 读取文件头 magic number ("TEXV")
2. 解析版本和格式标志
3. 读取图像尺寸
4. 解析 mipmap 结构
5. 判断数据格式和压缩状态

**错误**:
- `CoreError::NotFound`: 文件不存在
- `CoreError::Parse`: 不是有效的 TEX 文件
- `CoreError::Io`: 读取失败

### `convert_tex`

- **签名**: `fn convert_tex(input: ConvertTexInput) -> CoreResult<ConvertTexOutput>`
- **功能**: 将 TEX 文件转换为标准格式

**支持的输入格式**:

| 格式             | 说明                        | 输出     |
| ---------------- | --------------------------- | -------- |
| DXT1             | 压缩纹理 (4bpp)             | PNG      |
| DXT3             | 压缩纹理 (8bpp, 显式 alpha) | PNG      |
| DXT5             | 压缩纹理 (8bpp, 插值 alpha) | PNG      |
| RGBA8888         | 原始 RGBA                   | PNG      |
| RG88             | 双通道                      | PNG      |
| R8               | 单通道                      | PNG      |
| PNG/JPEG/BMP/GIF | 已是图片                    | 直接复制 |
| MP4              | 视频                        | 直接复制 |

**处理流程**:
1. 调用 `parse_tex` 获取格式信息
2. 根据格式选择解码器：
   - 压缩格式 → DXT 解码 → PNG
   - 原始格式 → 重组像素 → PNG
   - 图片格式 → 直接复制
   - 视频格式 → 直接复制
3. 如果 LZ4 压缩，先解压
4. 输出到目标路径

**错误**:
- `CoreError::NotFound`: TEX 文件不存在
- `CoreError::Parse`: 解析失败
- `CoreError::Unsupported`: 不支持的格式
- `CoreError::Io`: 写入输出文件失败

## TEX 文件格式

### 文件结构

```
┌─────────────────────────────────────────┐
│ Header                                  │
│  - Magic: "TEXV" (4 bytes)              │
│  - Version/Flags (4 bytes)              │
│  - Format (4 bytes)                     │
│  - Texture size (8 bytes)               │
│  - Image size (8 bytes)                 │
├─────────────────────────────────────────┤
│ Image array                             │
│  每个 Image:                             │
│  - Image format: 4 bytes                │
│  - Mipmap count: 4 bytes                │
│  └─ Mipmap array                        │
│      每个 Mipmap:                        │
│      - Width: 4 bytes                   │
│      - Height: 4 bytes                  │
│      - LZ4 flag: 1 byte                 │
│      - Decompressed size: 4 bytes       │
│      - Data size: 4 bytes               │
│      - Data: N bytes                    │
└─────────────────────────────────────────┘
```

### 格式详解

#### DXT 压缩格式

| 格式 | 压缩率 | Alpha      | 适用场景         |
| ---- | ------ | ---------- | ---------------- |
| DXT1 | 8:1    | 1-bit      | 不透明或简单透明 |
| DXT3 | 4:1    | 显式 4-bit | 锐利边缘透明     |
| DXT5 | 4:1    | 插值 8-bit | 平滑渐变透明     |

#### 原始格式

| 格式     | 每像素字节 | 说明                  |
| -------- | ---------- | --------------------- |
| RGBA8888 | 4          | 标准 RGBA             |
| RG88     | 2          | 双通道（法线贴图等）  |
| R8       | 1          | 单通道（灰度/高度图） |

## 使用示例

### 预览 TEX 文件

```rust
use crate::core::tex::*;
use std::path::PathBuf;

let result = parse_tex(ParseTexInput {
    file_path: PathBuf::from("/path/to/texture.tex"),
})?;

println!("TEX 信息:");
println!("  版本: {}", result.tex_info.version);
println!("  格式: {}", result.tex_info.format);
println!("  尺寸: {}x{}", result.tex_info.width, result.tex_info.height);
println!("  图像数: {}", result.tex_info.image_count);
println!("  Mipmap数: {}", result.tex_info.mipmap_count);
println!("  LZ4压缩: {}", result.tex_info.is_compressed);
println!("  视频: {}", result.tex_info.is_video);
```

### 转换单个文件

```rust
let result = convert_tex(ConvertTexInput {
    file_path: PathBuf::from("/path/to/texture.tex"),
    output_path: PathBuf::from("/output/texture.png"),
})?;

println!("转换完成:");
println!("  输出: {:?}", result.converted_file.output_path);
println!("  格式: {}", result.converted_file.format);
println!("  尺寸: {}x{}", result.converted_file.width, result.converted_file.height);
```

### 批量转换

```rust
use crate::core::path::{scan_files, ScanFilesInput};

// 扫描所有 .tex 文件
let scan_result = scan_files(ScanFilesInput {
    path: PathBuf::from("/unpacked/scene"),
    extensions: Some(vec!["tex".to_string()]),
})?;

for tex_file in scan_result.files {
    // 构建输出路径（.tex → .png）
    let output_path = tex_file.with_extension("png");
    
    match convert_tex(ConvertTexInput {
        file_path: tex_file.clone(),
        output_path: output_path.clone(),
    }) {
        Ok(result) => {
            println!("✓ {:?} -> {:?}", tex_file, result.converted_file.output_path);
        }
        Err(e) => {
            eprintln!("✗ {:?}: {}", tex_file, e);
        }
    }
}
```

### 按格式分类处理

```rust
let result = parse_tex(ParseTexInput {
    file_path: PathBuf::from("/path/to/texture.tex"),
})?;

match result.tex_info.format.as_str() {
    "DXT1" | "DXT3" | "DXT5" => {
        println!("压缩纹理，需要解码");
    }
    "RGBA8888" | "RG88" | "R8" => {
        println!("原始格式，直接重组");
    }
    "PNG" | "JPEG" | "BMP" | "GIF" => {
        println!("已是图片，直接复制");
    }
    "MP4" => {
        println!("视频文件，直接复制");
    }
    _ => {
        println!("未知格式: {}", result.tex_info.format);
    }
}
```

## 设计说明

### 为什么图片/视频直接复制?

TEX 文件可以直接封装标准图片（PNG、JPEG 等）或视频（MP4）。这种情况下数据已是最终格式，无需转换，直接复制效率最高。

### Mipmap 处理策略

TEX 文件通常包含多级 mipmap（缩小版本用于远距离渲染）。转换时只取最大的一级（mipmap[0]），忽略其他级别。

### LZ4 解压

部分 TEX 文件的 mipmap 数据经过 LZ4 压缩。解码前需要先用 `decompressed_bytes_count` 分配缓冲区，然后调用 LZ4 解压。

### 与 paper/pkg 模块的关系

```
paper 模块          pkg 模块              tex 模块
    │                  │                     │
    ▼                  ▼                     ▼
复制 .pkg       →   解包 .tex 文件   →   转换为 .png
到临时目录            到解包目录            或 .mp4
```

tex 模块是整个流程的最后一步，将解包得到的 .tex 文件转换为用户可用的标准图片或视频格式。
