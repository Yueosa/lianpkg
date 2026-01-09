# Core 模块文档

## 概述

`core` 模块是 lianpkg 的核心业务逻辑层，负责 Wallpaper Engine 壁纸的扫描、解包和转换。

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                         外部接口                             │
│              (CLI / API Native / FFI)                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        core 模块                            │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │  paper  │→ │   pkg   │→ │   tex   │→ │   cfg   │        │
│  │ (扫描)  │  │ (解包)  │  │ (转换)  │  │ (配置)  │        │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘        │
│       │            │            │            │              │
│       └────────────┴────────────┴────────────┘              │
│                         │                                   │
│                    ┌────┴────┐                              │
│                    │  path   │                              │
│                    │ (路径)  │                              │
│                    └────┬────┘                              │
│                         │                                   │
│                    ┌────┴────┐                              │
│                    │  error  │                              │
│                    │ (错误)  │                              │
│                    └─────────┘                              │
└─────────────────────────────────────────────────────────────┘
```

## 模块列表

| 模块              | 职责           | 接口数   | 文档                                                                   |
| ----------------- | -------------- | -------- | ---------------------------------------------------------------------- |
| [error](error.md) | 统一错误处理   | 2 (类型) | CoreError, CoreResult                                                  |
| [path](path.md)   | 路径工具       | 4        | expand_path, ensure_dir, resolve_path, scan_files                      |
| [cfg](cfg.md)     | 配置管理       | 9        | config.toml (4) + state.json (4) + clear (1)                           |
| [paper](paper.md) | 壁纸扫描与复制 | 6        | list_dirs, read_meta, check_pkg, estimate, process_folder, extract_all |
| [pkg](pkg.md)     | PKG 解包       | 3        | parse_pkg, unpack_pkg, unpack_entry                                    |
| [tex](tex.md)     | TEX 转换       | 2        | parse_tex, convert_tex                                                 |

## 处理流程

### 完整流程

```
1. paper.extract_all()
   ├── 扫描 Steam Workshop 目录
   ├── 识别壁纸类型 (原始/PKG)
   └── 复制文件到临时目录

2. pkg.unpack_pkg()
   ├── 解析 PKG 文件结构
   └── 解包到输出目录

3. tex.convert_tex()
   ├── 解析 TEX 格式
   ├── DXT 解码 / 像素重组 / 直接复制
   └── 输出 PNG/MP4

4. cfg.write_state_json()
   └── 记录处理状态
```

### 流程图

```
┌────────────────┐     ┌────────────────┐     ┌────────────────┐
│ Steam Workshop │     │   PKG 文件     │     │   TEX 文件     │
│   /431960/     │     │   scene.pkg    │     │  texture.tex   │
└───────┬────────┘     └───────┬────────┘     └───────┬────────┘
        │                      │                      │
        ▼                      ▼                      ▼
   paper 模块             pkg 模块               tex 模块
   ┌────────┐            ┌────────┐            ┌────────┐
   │list_dirs│           │parse_pkg│           │parse_tex│
   │check_pkg│           │unpack   │           │convert  │
   │process  │           │         │           │         │
   └────┬────┘           └────┬────┘           └────┬────┘
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│ 临时目录      │     │ 解包目录      │     │ 最终输出      │
│ /tmp/lianpkg │ ──→ │ ~/unpacked/  │ ──→ │ texture.png  │
│ scene.pkg    │     │ texture.tex  │     │ video.mp4    │
└───────────────┘     └───────────────┘     └───────────────┘
```

## 设计原则

### 1. Input/Output 结构体

所有公开接口使用 Input/Output 结构体模式：

```rust
// ✓ 正确
fn process_folder(input: ProcessFolderInput) -> CoreResult<ProcessFolderOutput>

// ✗ 避免
fn process_folder(path: &Path, config: &Config) -> Result<Stats, Error>
```

**优点**:
- 便于序列化（FFI 传递）
- 参数扩展不破坏 API
- 自文档化

### 2. 统一错误类型

所有接口返回 `CoreResult<T>` = `Result<T, CoreError>`：

```rust
pub enum CoreError {
    Io { message, path },
    Parse { message, source },
    Validation { message },
    NotFound { message, path },
    Unsupported { message },
}
```

**优点**:
- 可序列化（跨 FFI）
- 可模式匹配
- 可 Clone

### 3. 模块独立性

各模块可独立使用：

```rust
// 只用 tex 模块
use lianpkg::core::tex::{parse_tex, convert_tex, ParseTexInput, ConvertTexInput};

// 只用 pkg 模块
use lianpkg::core::pkg::{parse_pkg, unpack_pkg, ParsePkgInput, UnpackPkgInput};
```

### 4. 跨平台支持

- Linux: `~/.config/lianpkg`, `/tmp/lianpkg_temp`
- Windows: `%APPDATA%\lianpkg`, `%TEMP%\lianpkg_temp`

通过 `path` 模块的 `resolve_path()` 统一处理。

## 快速开始

### 一键提取壁纸

```rust
use lianpkg::core::paper::{extract_all, ExtractInput, PaperConfig};
use lianpkg::core::pkg::{unpack_pkg, UnpackPkgInput};
use lianpkg::core::tex::{convert_tex, ConvertTexInput};
use lianpkg::core::path::{scan_files, ScanFilesInput};
use std::path::PathBuf;

// 1. 扫描并复制
let extract_result = extract_all(ExtractInput {
    config: PaperConfig {
        search_path: PathBuf::from("/path/to/workshop/431960"),
        raw_output: PathBuf::from("/output/raw"),
        pkg_temp_output: PathBuf::from("/tmp/lianpkg_temp"),
        enable_raw: true,
    },
})?;

// 2. 解包 PKG
for folder in extract_result.processed_folders {
    for pkg_file in folder.pkg_files {
        let unpack_result = unpack_pkg(UnpackPkgInput {
            file_path: pkg_file,
            output_base: PathBuf::from("/output/unpacked"),
        })?;
        
        // 3. 转换 TEX
        let tex_files = scan_files(ScanFilesInput {
            path: PathBuf::from("/output/unpacked"),
            extensions: Some(vec!["tex".to_string()]),
        })?;
        
        for tex_file in tex_files.files {
            let output = tex_file.with_extension("png");
            convert_tex(ConvertTexInput {
                file_path: tex_file,
                output_path: output,
            })?;
        }
    }
}
```

## 文件列表

```
docs/core/
├── README.md       # 本文件 - 模块概览
├── error.md        # 错误处理
├── path.md         # 路径工具
├── cfg.md          # 配置管理
├── paper.md        # 壁纸扫描与复制
├── pkg.md          # PKG 解包
└── tex.md          # TEX 转换
```
