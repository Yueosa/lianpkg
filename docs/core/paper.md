# core::paper 模块

壁纸扫描与提取模块。负责 Workshop 目录的遍历、元数据读取、PKG 文件检测、空间估算和壁纸提取。

## 文件结构

| 文件 | 职责 |
|------|------|
| `mod.rs` | 模块声明与公开导出 |
| `structs.rs` | 所有结构体定义（Config / Input / Output / Runtime） |
| `scan.rs` | 扫描接口：`list_dirs`、`read_meta`、`check_pkg`、`estimate` |
| `copy.rs` | 提取接口：`process_folder`、`extract_all` |
| `utl.rs` | 内部工具：`link_or_copy`、`copy_dir_recursive`、`get_dir_size` |

## Step 6 变更摘要

### 6.1 新增 `link_or_copy(src, dst)`

```rust
pub(crate) fn link_or_copy(src: &Path, dst: &Path) -> io::Result<()>
```

- **Linux (unix)**：优先 `hard_link`（零额外空间、瞬时完成），跨文件系统失败时自动 fallback 到 `fs::copy`
- **Windows (非 unix)**：直接 `fs::copy`
- 被 `copy_dir_recursive()` 调用，替换原来的 `fs::copy`，使 raw 壁纸提取在同文件系统时零额外空间

### 6.2 消除 Pkg_Temp 中间目录

**旧流转**（双倍 I/O + 额外磁盘占用）：
```
Workshop/{id}/*.pkg ──copy──▶ Pkg_Temp/{id}_*.pkg ──read+unpack──▶ Pkg_Unpacked/{id}/
```

**新流转**（直接读取 Workshop）：
```
Workshop/{id}/*.pkg ──直接 read+unpack──▶ Pkg_Unpacked/{id}/
```

具体变更：

| 项 | 旧 | 新 |
|---|---|---|
| `PaperConfig` | 含 `pkg_temp_output: PathBuf` | **已移除** |
| `ProcessFolderInput` | 含 `pkg_temp_output: PathBuf` | **已移除** |
| PKG 分支逻辑 | 复制 PKG 文件到 Pkg_Temp 目录 | **不复制**，直接返回 Workshop 源路径 |
| `ProcessFolderOutput.pkg_files` | Pkg_Temp 中的目标路径 | Workshop 中的 **源路径** |
| `extract_all()` | 传递 `pkg_temp_output` | 不再需要 |

### 6.3 `process_folder()` 新行为

```rust
pub fn process_folder(input: ProcessFolderInput) -> ProcessFolderOutput
```

- **含 PKG 的文件夹**：返回 `ProcessResultType::Pkg`，`pkg_files` 为 Workshop 中 `.pkg` 文件的绝对路径列表。不执行任何文件复制。后续由 api::native::pkg 阶段直接从这些路径读取解包。
- **不含 PKG 且 `enable_raw`**：使用 `copy_dir_recursive`（内部 `link_or_copy`）将整个目录提取到 `raw_output/{id}/`。目标已存在时跳过。
- **其他情况**：返回 `Skipped`。

## 公开接口

### 扫描

| 函数 | 签名 | 说明 |
|------|------|------|
| `list_dirs` | `(ListDirsInput) → CoreResult<ListDirsOutput>` | 列出所有子目录 |
| `read_meta` | `(ReadMetaInput) → CoreResult<ReadMetaOutput>` | 读取 `project.json` |
| `check_pkg` | `(CheckPkgInput) → CheckPkgOutput` | 检测 `.pkg` 文件 |
| `estimate` | `(EstimateInput) → EstimateOutput` | 估算磁盘空间 |

### 提取

| 函数 | 签名 | 说明 |
|------|------|------|
| `process_folder` | `(ProcessFolderInput) → ProcessFolderOutput` | 处理单个壁纸目录 |
| `extract_all` | `(ExtractInput) → ExtractOutput` | 批量遍历并处理 |

## 关键结构体

```rust
pub struct PaperConfig {
    pub search_path: PathBuf,   // Workshop 搜索路径
    pub raw_output: PathBuf,    // 原始壁纸输出路径
    pub enable_raw: bool,       // 是否提取原始壁纸
}

pub struct ProcessFolderOutput {
    pub copied_raw: bool,
    pub copied_pkgs: usize,       // PKG 文件数（不再实际复制）
    pub skipped: bool,
    pub result_type: ProcessResultType,
    pub pkg_files: Vec<PathBuf>,  // Workshop 源路径列表
}

pub enum ProcessResultType { Raw, Pkg, Skipped }
```
