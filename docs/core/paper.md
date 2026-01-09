# paper 模块

## 概述

- **模块职责**: 扫描 Wallpaper Engine 壁纸目录，识别并复制壁纸文件（原始壁纸或 PKG 文件）
- **使用场景**: 
  - 一键提取：直接调用 `extract_all` 
  - 精细控制：组合 `list_dirs` → `check_pkg` → `process_folder`
- **依赖关系**: 依赖 `error`、`path` 模块

## 接口列表

| 函数             | 输入                 | 输出                  | 说明                     |
| ---------------- | -------------------- | --------------------- | ------------------------ |
| `list_dirs`      | `ListDirsInput`      | `ListDirsOutput`      | 列出壁纸目录             |
| `read_meta`      | `ReadMetaInput`      | `ReadMetaOutput`      | 读取 project.json 元数据 |
| `check_pkg`      | `CheckPkgInput`      | `CheckPkgOutput`      | 检查是否包含 PKG 文件    |
| `estimate`       | `EstimateInput`      | `EstimateOutput`      | 估算处理结果             |
| `process_folder` | `ProcessFolderInput` | `ProcessFolderOutput` | 处理单个文件夹           |
| `extract_all`    | `ExtractInput`       | `ExtractOutput`       | 一键提取所有壁纸         |

## 数据结构

### 配置结构体

#### PaperConfig
```rust
pub struct PaperConfig {
    /// workshop 搜索路径
    pub search_path: PathBuf,
    /// 原始壁纸输出路径
    pub raw_output: PathBuf,
    /// pkg 临时输出路径
    pub pkg_temp_output: PathBuf,
    /// 是否提取原始壁纸
    pub enable_raw: bool,
}
```

### Input 结构体

#### ListDirsInput
```rust
pub struct ListDirsInput {
    /// 搜索路径（Steam Workshop content 目录）
    pub path: PathBuf,
}
```

#### ReadMetaInput
```rust
pub struct ReadMetaInput {
    /// 壁纸文件夹路径
    pub folder: PathBuf,
}
```

#### CheckPkgInput
```rust
pub struct CheckPkgInput {
    /// 壁纸文件夹路径
    pub folder: PathBuf,
}
```

#### EstimateInput
```rust
pub struct EstimateInput {
    /// 搜索路径
    pub search_path: PathBuf,
    /// 是否计算原始壁纸大小
    pub enable_raw: bool,
}
```

#### ProcessFolderInput
```rust
pub struct ProcessFolderInput {
    /// 壁纸文件夹路径
    pub folder: PathBuf,
    /// 原始壁纸输出路径
    pub raw_output: PathBuf,
    /// pkg 临时输出路径
    pub pkg_temp_output: PathBuf,
    /// 是否提取原始壁纸
    pub enable_raw: bool,
}
```

#### ExtractInput
```rust
pub struct ExtractInput {
    /// 运行配置
    pub config: PaperConfig,
}
```

### Output 结构体

#### ListDirsOutput
```rust
pub struct ListDirsOutput {
    /// 目录名列表（workshop ID）
    pub dirs: Vec<String>,
}
```

#### ReadMetaOutput
```rust
pub struct ReadMetaOutput {
    /// 壁纸元数据
    pub meta: ProjectMeta,
}
```

#### CheckPkgOutput
```rust
pub struct CheckPkgOutput {
    /// 是否包含 pkg 文件
    pub has_pkg: bool,
    /// pkg 文件路径列表
    pub pkg_files: Vec<PathBuf>,
}
```

#### EstimateOutput
```rust
pub struct EstimateOutput {
    /// pkg 文件总大小（字节）
    pub pkg_size: u64,
    /// 原始壁纸总大小（字节）
    pub raw_size: u64,
    /// pkg 壁纸数量
    pub pkg_count: usize,
    /// 原始壁纸数量
    pub raw_count: usize,
}
```

#### ProcessFolderOutput
```rust
pub struct ProcessFolderOutput {
    /// 是否复制了原始壁纸
    pub copied_raw: bool,
    /// 复制的 pkg 文件数量
    pub copied_pkgs: usize,
    /// 是否跳过（已存在等原因）
    pub skipped: bool,
    /// 处理结果类型
    pub result_type: ProcessResultType,
    /// 复制的 pkg 文件路径列表
    pub pkg_files: Vec<PathBuf>,
}
```

#### ExtractOutput
```rust
pub struct ExtractOutput {
    /// 统计信息
    pub stats: WallpaperStats,
    /// 处理的文件夹详情列表
    pub processed_folders: Vec<ProcessedFolder>,
}
```

### 运行时结构体

#### ProjectMeta
```rust
pub struct ProjectMeta {
    pub contentrating: Option<String>,
    pub description: Option<String>,
    pub file: Option<String>,       // 主文件名
    pub preview: Option<String>,    // 预览图
    pub tags: Option<Vec<String>>,
    pub title: Option<String>,      // 壁纸标题
    pub wallpaper_type: Option<String>,  // "video", "scene" 等
    pub version: Option<u32>,
    pub workshopid: Option<String>,
    pub workshopurl: Option<String>,
    pub general: Option<serde_json::Value>,
}
```

#### WallpaperStats
```rust
pub struct WallpaperStats {
    pub raw_count: usize,
    pub pkg_count: usize,
    pub total_size: u64,  // 字节
}
```

#### ProcessedFolder
```rust
pub struct ProcessedFolder {
    pub folder_name: String,
    pub folder_path: PathBuf,
    pub result_type: ProcessResultType,
    pub pkg_files: Vec<PathBuf>,
}

pub enum ProcessResultType {
    Raw,      // 复制为原始壁纸
    Pkg,      // 复制了 pkg 文件
    Skipped,  // 跳过
}
```

## 接口详解

### `list_dirs`

- **签名**: `fn list_dirs(input: ListDirsInput) -> CoreResult<ListDirsOutput>`
- **功能**: 列出指定路径下的所有子目录（壁纸文件夹）

**处理流程**:
1. 读取目录内容
2. 过滤出子目录
3. 返回目录名列表（通常是 workshop ID）

**错误**:
- `CoreError::NotFound`: 路径不存在
- `CoreError::Io`: 读取目录失败

### `read_meta`

- **签名**: `fn read_meta(input: ReadMetaInput) -> CoreResult<ReadMetaOutput>`
- **功能**: 读取壁纸文件夹中的 `project.json`

**处理流程**:
1. 拼接 `folder/project.json` 路径
2. 读取并解析 JSON
3. 返回 `ProjectMeta` 结构

**错误**:
- `CoreError::NotFound`: project.json 不存在
- `CoreError::Parse`: JSON 解析失败

### `check_pkg`

- **签名**: `fn check_pkg(input: CheckPkgInput) -> CoreResult<CheckPkgOutput>`
- **功能**: 检查壁纸文件夹是否包含 PKG 文件

**处理流程**:
1. 扫描文件夹中所有 `.pkg` 文件
2. 返回是否有 PKG 及文件列表

**典型用途**: 判断壁纸类型（原始 vs PKG 打包）

### `estimate`

- **签名**: `fn estimate(input: EstimateInput) -> CoreResult<EstimateOutput>`
- **功能**: 预估处理所需的磁盘空间和文件数量

**返回信息**:
- PKG 文件总大小和数量
- 原始壁纸总大小和数量（如果 enable_raw）

**典型用途**: 在处理前向用户显示预估信息

### `process_folder`

- **签名**: `fn process_folder(input: ProcessFolderInput) -> CoreResult<ProcessFolderOutput>`
- **功能**: 处理单个壁纸文件夹

**处理流程**:
1. 检查是否已存在于输出目录
2. 如果有 PKG 文件，复制到 `pkg_temp_output`
3. 如果是原始壁纸且 `enable_raw`，复制到 `raw_output`
4. 返回处理结果

**错误**:
- `CoreError::Io`: 复制文件失败

### `extract_all`

- **签名**: `fn extract_all(input: ExtractInput) -> CoreResult<ExtractOutput>`
- **功能**: 一键提取所有壁纸

**处理流程**:
1. 调用 `list_dirs` 获取所有目录
2. 遍历每个目录调用 `process_folder`
3. 汇总统计信息

## 使用示例

### 一键提取

```rust
use crate::core::paper::*;
use std::path::PathBuf;

let result = extract_all(ExtractInput {
    config: PaperConfig {
        search_path: PathBuf::from("/path/to/workshop/431960"),
        raw_output: PathBuf::from("/output/raw"),
        pkg_temp_output: PathBuf::from("/tmp/lianpkg_temp"),
        enable_raw: true,
    },
})?;

println!("处理完成：");
println!("  原始壁纸: {} 个", result.stats.raw_count);
println!("  PKG 壁纸: {} 个", result.stats.pkg_count);
println!("  总大小: {} bytes", result.stats.total_size);
```

### 精细控制

```rust
use crate::core::paper::*;
use std::path::PathBuf;

let workshop_path = PathBuf::from("/path/to/workshop/431960");

// 1. 列出所有壁纸目录
let dirs = list_dirs(ListDirsInput {
    path: workshop_path.clone(),
})?;

for dir_name in dirs.dirs {
    let folder = workshop_path.join(&dir_name);
    
    // 2. 读取元数据
    if let Ok(meta_result) = read_meta(ReadMetaInput {
        folder: folder.clone(),
    }) {
        println!("壁纸: {}", meta_result.meta.title.unwrap_or_default());
    }
    
    // 3. 检查是否有 PKG
    let check_result = check_pkg(CheckPkgInput {
        folder: folder.clone(),
    })?;
    
    if check_result.has_pkg {
        println!("  类型: PKG, 文件数: {}", check_result.pkg_files.len());
    } else {
        println!("  类型: 原始壁纸");
    }
    
    // 4. 处理文件夹
    let process_result = process_folder(ProcessFolderInput {
        folder,
        raw_output: PathBuf::from("/output/raw"),
        pkg_temp_output: PathBuf::from("/tmp/lianpkg_temp"),
        enable_raw: true,
    })?;
    
    match process_result.result_type {
        ProcessResultType::Pkg => {
            // PKG 文件已复制，后续可调用 pkg 模块解包
            for pkg_file in process_result.pkg_files {
                println!("  复制 PKG: {:?}", pkg_file);
            }
        }
        ProcessResultType::Raw => {
            println!("  已复制原始壁纸");
        }
        ProcessResultType::Skipped => {
            println!("  已跳过");
        }
    }
}
```

### 预估空间

```rust
let estimate_result = estimate(EstimateInput {
    search_path: PathBuf::from("/path/to/workshop/431960"),
    enable_raw: true,
})?;

println!("预估结果:");
println!("  PKG 壁纸: {} 个, 约 {} MB", 
    estimate_result.pkg_count,
    estimate_result.pkg_size / 1024 / 1024);
println!("  原始壁纸: {} 个, 约 {} MB",
    estimate_result.raw_count,
    estimate_result.raw_size / 1024 / 1024);
```

## 设计说明

### 壁纸类型判断逻辑

1. 文件夹包含 `.pkg` 文件 → PKG 壁纸
2. 否则 → 原始壁纸（直接包含图片/视频/场景文件）

### 为什么分离扫描和复制?

- **扫描接口** (`list_dirs`, `read_meta`, `check_pkg`): 只读操作，用于预览和决策
- **复制接口** (`process_folder`, `extract_all`): 写操作，实际执行提取

这种分离允许 GUI 先显示列表让用户选择，再执行提取。

### 与 pkg/tex 模块的关系

```
paper 模块              pkg 模块              tex 模块
    │                      │                     │
    ▼                      ▼                     ▼
扫描 workshop      →   解包 .pkg 文件    →   转换 .tex 文件
复制到临时目录           输出原始文件           输出图片/视频
```

paper 模块是流程的第一步，将 PKG 文件复制到临时目录后，由 pkg 模块解包，再由 tex 模块转换。
