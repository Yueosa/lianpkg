# pkg 模块

## 概述

- **模块职责**: 解析和解包 Wallpaper Engine 的 PKG 打包文件
- **使用场景**: 
  - 预览：`parse_pkg` 查看 PKG 内容列表
  - 一键解包：`unpack_pkg` 解包整个 PKG
  - 选择性解包：`parse_pkg` → 判断 → `unpack_entry` 解包指定条目
- **依赖关系**: 依赖 `error`、`path` 模块

## 接口列表

| 函数           | 输入               | 输出                | 说明              |
| -------------- | ------------------ | ------------------- | ----------------- |
| `parse_pkg`    | `ParsePkgInput`    | `ParsePkgOutput`    | 解析 PKG 文件结构 |
| `unpack_pkg`   | `UnpackPkgInput`   | `UnpackPkgOutput`   | 一键解包整个 PKG  |
| `unpack_entry` | `UnpackEntryInput` | `UnpackEntryOutput` | 解包单个条目      |

## 数据结构

### Input 结构体

#### ParsePkgInput
```rust
pub struct ParsePkgInput {
    /// pkg 文件路径
    pub file_path: PathBuf,
}
```

#### UnpackPkgInput
```rust
pub struct UnpackPkgInput {
    /// pkg 文件路径
    pub file_path: PathBuf,
    /// 输出目录
    pub output_base: PathBuf,
}
```

#### UnpackEntryInput
```rust
pub struct UnpackEntryInput {
    /// pkg 文件原始数据
    pub pkg_data: Vec<u8>,
    /// 数据区起始偏移
    pub data_start: usize,
    /// 要解包的条目
    pub entry: PkgEntry,
    /// 输出路径
    pub output_path: PathBuf,
}
```

### Output 结构体

#### ParsePkgOutput
```rust
pub struct ParsePkgOutput {
    /// pkg 文件信息
    pub pkg_info: PkgInfo,
}
```

#### UnpackPkgOutput
```rust
pub struct UnpackPkgOutput {
    /// pkg 文件信息
    pub pkg_info: PkgInfo,
    /// 解包的文件列表
    pub extracted_files: Vec<ExtractedFile>,
}
```

#### UnpackEntryOutput
```rust
pub struct UnpackEntryOutput {
    /// 输出路径
    pub output_path: PathBuf,
}
```

### 运行时结构体

#### PkgInfo
```rust
pub struct PkgInfo {
    /// pkg 版本字符串
    pub version: String,
    /// 文件数量
    pub file_count: u32,
    /// 文件条目列表
    pub entries: Vec<PkgEntry>,
    /// 数据区起始偏移
    pub data_start: usize,
}
```

#### PkgEntry
```rust
pub struct PkgEntry {
    /// 文件名（含路径，如 "materials/texture.tex"）
    pub name: String,
    /// 在数据区中的偏移
    pub offset: u32,
    /// 文件大小
    pub size: u32,
}
```

#### ExtractedFile
```rust
pub struct ExtractedFile {
    /// 原始条目名
    pub entry_name: String,
    /// 输出路径
    pub output_path: PathBuf,
}
```

## 接口详解

### `parse_pkg`

- **签名**: `fn parse_pkg(input: ParsePkgInput) -> CoreResult<ParsePkgOutput>`
- **功能**: 解析 PKG 文件头，获取文件列表

**处理流程**:
1. 读取文件头 magic number
2. 解析版本号
3. 读取文件数量
4. 逐个读取文件条目（名称、偏移、大小）
5. 计算数据区起始位置

**错误**:
- `CoreError::NotFound`: 文件不存在
- `CoreError::Parse`: 不是有效的 PKG 文件
- `CoreError::Io`: 读取失败

### `unpack_pkg`

- **签名**: `fn unpack_pkg(input: UnpackPkgInput) -> CoreResult<UnpackPkgOutput>`
- **功能**: 解包整个 PKG 文件

**处理流程**:
1. 调用 `parse_pkg` 解析文件结构
2. 创建输出目录
3. 遍历所有条目，提取文件
4. 保持原有目录结构

**输出目录结构**:
```
output_base/
└── scene_name/
    ├── project.json
    ├── scene.json
    └── materials/
        ├── texture1.tex
        └── texture2.tex
```

**错误**:
- `CoreError::NotFound`: PKG 文件不存在
- `CoreError::Parse`: 解析失败
- `CoreError::Io`: 创建目录或写入文件失败

### `unpack_entry`

- **签名**: `fn unpack_entry(input: UnpackEntryInput) -> CoreResult<UnpackEntryOutput>`
- **功能**: 解包单个文件条目

**典型用途**: 
- 只需要某些特定文件（如只解包 .tex 文件）
- 实现进度回调（逐个解包并报告进度）

**处理流程**:
1. 根据 entry 的 offset 和 size 定位数据
2. 从 pkg_data 中提取字节
3. 创建输出目录并写入文件

**错误**:
- `CoreError::Io`: 写入失败

## PKG 文件格式

### 文件结构

```
┌────────────────────────────────────────┐
│ Header (可变长度)                       │
│  - Magic: "PKGV" (4 bytes)             │
│  - Version: 4 bytes (little-endian)    │
│  - File count: 4 bytes (little-endian) │
├────────────────────────────────────────┤
│ File entries (可变长度)                 │
│  每个条目:                              │
│  - Name length: 4 bytes                │
│  - Name: N bytes (UTF-8)               │
│  - Offset: 4 bytes                     │
│  - Size: 4 bytes                       │
├────────────────────────────────────────┤
│ Data section (可变长度)                 │
│  - 按条目偏移排列的原始文件数据          │
└────────────────────────────────────────┘
```

### 支持的版本

| 版本 | 说明               |
| ---- | ------------------ |
| 1    | 早期版本，基础结构 |
| 2    | 当前常见版本       |

## 使用示例

### 预览 PKG 内容

```rust
use crate::core::pkg::*;
use std::path::PathBuf;

let result = parse_pkg(ParsePkgInput {
    file_path: PathBuf::from("/path/to/scene.pkg"),
})?;

println!("PKG 版本: {}", result.pkg_info.version);
println!("文件数量: {}", result.pkg_info.file_count);
println!("文件列表:");
for entry in &result.pkg_info.entries {
    println!("  {} ({} bytes)", entry.name, entry.size);
}
```

### 一键解包

```rust
let result = unpack_pkg(UnpackPkgInput {
    file_path: PathBuf::from("/path/to/scene.pkg"),
    output_base: PathBuf::from("/output"),
})?;

println!("解包完成：");
for file in result.extracted_files {
    println!("  {} -> {:?}", file.entry_name, file.output_path);
}
```

### 选择性解包（只解包 .tex 文件）

```rust
use std::fs;

// 1. 解析 PKG
let pkg_result = parse_pkg(ParsePkgInput {
    file_path: PathBuf::from("/path/to/scene.pkg"),
})?;

// 2. 读取完整文件数据
let pkg_data = fs::read("/path/to/scene.pkg")?;

// 3. 筛选 .tex 文件并解包
for entry in pkg_result.pkg_info.entries {
    if entry.name.ends_with(".tex") {
        let output_path = PathBuf::from("/output").join(&entry.name);
        
        unpack_entry(UnpackEntryInput {
            pkg_data: pkg_data.clone(),
            data_start: pkg_result.pkg_info.data_start,
            entry,
            output_path,
        })?;
    }
}
```

### 带进度回调的解包

```rust
let pkg_result = parse_pkg(ParsePkgInput {
    file_path: PathBuf::from("/path/to/scene.pkg"),
})?;

let pkg_data = fs::read("/path/to/scene.pkg")?;
let total = pkg_result.pkg_info.entries.len();

for (i, entry) in pkg_result.pkg_info.entries.iter().enumerate() {
    let output_path = PathBuf::from("/output").join(&entry.name);
    
    unpack_entry(UnpackEntryInput {
        pkg_data: pkg_data.clone(),
        data_start: pkg_result.pkg_info.data_start,
        entry: entry.clone(),
        output_path,
    })?;
    
    // 报告进度
    println!("进度: {}/{} ({}%)", i + 1, total, (i + 1) * 100 / total);
}
```

## 设计说明

### 为什么 unpack_entry 需要 pkg_data?

为了支持：
1. 选择性解包（不必解包全部）
2. 进度回调（逐个解包）
3. 内存映射优化（大文件场景）

如果只提供文件路径，每次解包单个条目都要重新打开文件，效率低下。

### 与 paper/tex 模块的关系

```
paper 模块                pkg 模块              tex 模块
    │                        │                     │
    ▼                        ▼                     ▼
复制 .pkg 到临时目录   →   解包提取 .tex 等  →  转换 .tex 为图片
                            ↓
                       保留原始结构
                       materials/xxx.tex
```

解包后保持原有目录结构，便于 tex 模块按相对路径处理。
