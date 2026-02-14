# core::tex 模块

> TEX 文件解析与转换。支持预览（只读元数据）和完整转换（输出 PNG/MP4/原始图片）两种模式。

## 文件结构

```
src/core/tex/
├── mod.rs       # 模块声明与导出
├── structs.rs   # Input/Output + 运行时结构体 + MipmapFormat 枚举
├── reader.rs    # TEX 二进制读取器
├── parse.rs     # 解析接口 (parse_tex)
├── convert.rs   # 转换接口 (convert_tex)
└── decoder.rs   # 格式解码器 (DXT1/3/5, RGBA, RG, R8)
```

## 2 个公开接口

| 函数 | 说明 |
|------|------|
| `parse_tex(ParseTexInput)` | 解析 TEX 文件，返回元数据（不写磁盘） |
| `convert_tex(ConvertTexInput)` | 解析 + 解码 + 保存为目标格式 |

---

## TEX 文件格式

```
┌─────────────────────────────────────────────────┐
│ Magic1:  "TEXV0005"  (16 bytes, \0 padded)      │
│ Magic2:  "TEXI0001"  (16 bytes, \0 padded)      │
│ Header:  format, flags, dimensions (7 × u32)     │
│ Magic3:  "TEXB000X"  (16 bytes, X=1/2/3/4)      │
│ image_count (i32)                                │
│ [V3+] image_format (i32, FreeImage 格式编号)     │
│ [V4]  is_video_mp4 (i32)                         │
│ ┌─────────────────────────────────────────────┐  │
│ │ Image[0]                                    │  │
│ │   mipmap_count (i32)                        │  │
│ │   ┌──────────────────────────────────────┐  │  │
│ │   │ Mipmap[0]                            │  │  │
│ │   │   [V4] param1, param2, condition_json│  │  │
│ │   │   width, height (u32)                │  │  │
│ │   │   [V2+] is_lz4, decompressed_size   │  │  │
│ │   │   byte_count (i32) + raw data        │  │  │
│ │   └──────────────────────────────────────┘  │  │
│ │   Mipmap[1] ...                             │  │
│ └─────────────────────────────────────────────┘  │
│ Image[1] ...                                     │
└─────────────────────────────────────────────────┘
```

### 支持的版本

| Container Magic | 版本 | 特殊字段 |
|-----------------|------|----------|
| `TEXB0001` | V1 | 无 image_format |
| `TEXB0002` | V2 | +LZ4 压缩字段 |
| `TEXB0003` | V3 | +image_format (FreeImage 枚举) |
| `TEXB0004` | V4 | +is_video_mp4, +condition_json |

V4 且非视频时，effective_version 降级为 V3。

---

## Reader (`reader.rs`)

### `read_fixed_string(reader, field_size)`

精确消费 `field_size` 字节，在已读字节中找第一个 `\0` 截取字符串。

| 旧 `read_n_string` | 新 `read_fixed_string` |
|---------------------|------------------------|
| 逐字节读到 `\0` 停止 | 一次性读 `field_size` 字节 |
| 不消费 padding | 始终精确消费 `field_size` 字节 |
| 可能导致后续偏移错位 | 偏移保证正确 |

### `read_null_terminated_string(reader, max_bytes)`

用于 V4 mipmap 的变长 `condition_json` 字段：
- 逐字节读取直到 `\0`
- 安全上限 4096 字节，超过报 `InvalidData` 错误
- 替代旧 `read_n_string(reader, 0)` 的无限读取行为

### 校验

| 字段 | 限制 | 旧行为 |
|------|------|--------|
| `image_count` | 0 ≤ n ≤ 1000 | 无检查 |
| `mipmap_count` | 0 ≤ n ≤ 100 | 无检查 |
| `byte_count` | ≥ 0 且 ≤ 512 MB | 无检查（负数导致 `usize` 溢出） |
| Magic 字段 | 必须匹配已知值 | 已有检查，保留 |

### 错误类型

所有读取错误统一为 `CoreResult`：
- `io::Error` → `CoreError::InvalidData`（通过 `read_err` 辅助函数转换）
- Magic 不匹配 → `CoreError::InvalidData`
- UTF-8 解码失败 → `CoreError::InvalidData`

---

## 解码器 (`decoder.rs`)

### `determine_format(tex_file, image) → MipmapFormat`

判断优先级：
1. `is_video_mp4` → `VideoMp4`
2. `flags & 32` (IsVideoTexture) → `VideoMp4`
3. `image_format >= 0` → FreeImage 格式映射（30+ 种图片格式）
4. fallback 到 `header.format` → DXT1/3/5, RGBA8888, RG88, R8

### `decode_mipmap(data, width, height, format) → CoreResult<Vec<u8>>`

| 格式 | 解码方式 |
|------|----------|
| DXT1 | `texture2ddecoder::decode_bc1` → RGBA u32 → bytes |
| DXT3 | `texture2ddecoder::decode_bc2` → RGBA u32 → bytes |
| DXT5 | `texture2ddecoder::decode_bc3` → RGBA u32 → bytes |
| RGBA8888 | 直接返回 |
| RG88 | 2 bytes → RGBA (B=0, A=255) |
| R8 | 1 byte → RGBA 灰度 (A=255) |

返回类型从 `Result<Vec<u8>, String>` 改为 `CoreResult<Vec<u8>>`，统一错误链。

---

## 转换流程 (`convert.rs`)

```
1. File::open → read_tex → TexFile
2. 取 first_image.first_mipmap
3. determine_format → MipmapFormat
4. LZ4 解压（如果 is_lz4_compressed）
5. 根据格式分支：
   - VideoMp4 / Image* → save_raw_data（原始字节写出）
   - DXT/RGBA/RG/R8   → decode_mipmap → save_as_png
```

## 对外接口无破坏性变更

`parse_tex` / `convert_tex` 的签名和返回类型均未改变。`api/native/tex.rs` 和 `cli/handlers/tex.rs` 无需任何修改。
