# core 模块概览

本模块包含 LianPkg 的所有核心功能实现，采用统一的 **Input/Output 结构体** 设计模式。

## 模块结构

```
src/core/
├── mod.rs      # 模块入口
├── path/       # 路径处理与解析
├── cfg/        # 配置文件与状态文件操作
├── paper/      # Wallpaper 壁纸扫描与复制
├── pkg/        # Pkg 文件解析与解包
└── tex/        # Tex 文件解析与转换
```

---

## path - 路径处理与解析

**文件结构**: `utl.rs`, `cfg.rs`, `steam.rs`, `output.rs`, `pkg.rs`, `tex.rs`, `scan.rs`

### 导出接口

| 分类     | 接口                           | 说明                                   |
| -------- | ------------------------------ | -------------------------------------- |
| 通用工具 | `ensure_dir`                   | 确保目录存在                           |
| 通用工具 | `expand_path`                  | 展开 `~` 路径                          |
| 通用工具 | `get_unique_output_path`       | 获取唯一输出路径                       |
| Config   | `default_config_dir`           | 默认配置目录                           |
| Config   | `default_config_toml_path`     | config.toml 路径                       |
| Config   | `default_state_json_path`      | state.json 路径                        |
| Steam    | `default_workshop_path`        | Workshop 路径（支持原生/Flatpak/Snap） |
| Output   | `default_raw_output_path`      | 原始壁纸输出路径                       |
| Output   | `default_pkg_temp_path`        | Pkg 临时路径                           |
| Output   | `default_unpacked_output_path` | 解包输出路径                           |
| Pkg      | `pkg_temp_dest`                | Pkg 临时目标名                         |
| Pkg      | `scene_name_from_pkg_stem`     | 从文件名提取场景名                     |
| Tex      | `resolve_tex_output_dir`       | Tex 输出目录解析                       |
| Scan     | `get_target_files`             | 获取目标文件列表                       |
| Scan     | `find_project_root`            | 查找项目根目录                         |

---

## cfg - 配置文件与状态文件操作

**文件结构**: `structs.rs`, `utl.rs`, `config.rs`, `state.rs`, `clear.rs`

### 导出接口

| 接口                 | Input               | Output               | 说明         |
| -------------------- | ------------------- | -------------------- | ------------ |
| `create_config_toml` | `CreateConfigInput` | `CreateConfigOutput` | 创建配置文件 |
| `read_config_toml`   | `ReadConfigInput`   | `ReadConfigOutput`   | 读取配置文件 |
| `update_config_toml` | `UpdateConfigInput` | `UpdateConfigOutput` | 更新配置项   |
| `delete_config_toml` | `DeleteConfigInput` | `DeleteConfigOutput` | 删除配置文件 |
| `create_state_json`  | `CreateStateInput`  | `CreateStateOutput`  | 创建状态文件 |
| `read_state_json`    | `ReadStateInput`    | `ReadStateOutput`    | 读取状态文件 |
| `write_state_json`   | `WriteStateInput`   | `WriteStateOutput`   | 覆写状态文件 |
| `delete_state_json`  | `DeleteStateInput`  | `DeleteStateOutput`  | 删除状态文件 |
| `clear_lianpkg`      | `ClearInput`        | `ClearOutput`        | 递归删除目录 |

---

## paper - Wallpaper 壁纸扫描与复制

**文件结构**: `structs.rs`, `scan.rs`, `copy.rs`, `utl.rs`

### 导出接口

| 接口             | Input                | Output                | 说明              |
| ---------------- | -------------------- | --------------------- | ----------------- |
| `list_dirs`      | `ListDirsInput`      | `ListDirsOutput`      | 列出目录          |
| `read_meta`      | `ReadMetaInput`      | `ReadMetaOutput`      | 读取 project.json |
| `check_pkg`      | `CheckPkgInput`      | `CheckPkgOutput`      | 检查是否含 pkg    |
| `estimate`       | `EstimateInput`      | `EstimateOutput`      | 估算空间需求      |
| `process_folder` | `ProcessFolderInput` | `ProcessFolderOutput` | 处理单个文件夹    |
| `extract_all`    | `ExtractInput`       | `ExtractOutput`       | 批量提取          |

### 运行时结构体

- `PaperConfig` - 模块运行配置
- `ProjectMeta` - project.json 元数据
- `WallpaperStats` - 壁纸统计信息
- `ProcessedFolder` - 处理结果详情
- `ProcessResultType` - 处理结果类型枚举

---

## pkg - Pkg 文件解析与解包

**文件结构**: `structs.rs`, `parse.rs`, `unpack.rs`, `utl.rs`

### 导出接口

| 接口           | Input              | Output              | 说明                 |
| -------------- | ------------------ | ------------------- | -------------------- |
| `parse_pkg`    | `ParsePkgInput`    | `ParsePkgOutput`    | 解析元数据（不写入） |
| `unpack_pkg`   | `UnpackPkgInput`   | `UnpackPkgOutput`   | 解析并解包           |
| `unpack_entry` | `UnpackEntryInput` | `UnpackEntryOutput` | 解包单个条目         |

### 运行时结构体

- `PkgInfo` - Pkg 文件信息
- `PkgEntry` - 文件条目
- `ExtractedFile` - 解包后的文件信息

---

## tex - Tex 文件解析与转换

**文件结构**: `structs.rs`, `parse.rs`, `convert.rs`, `reader.rs`, `decoder.rs`

### 导出接口

| 接口          | Input             | Output             | 说明                 |
| ------------- | ----------------- | ------------------ | -------------------- |
| `parse_tex`   | `ParseTexInput`   | `ParseTexOutput`   | 解析元数据（不转换） |
| `convert_tex` | `ConvertTexInput` | `ConvertTexOutput` | 解析并转换保存       |

### 运行时结构体

- `TexInfo` - Tex 文件信息
- `ConvertedFile` - 转换后的文件信息
- `MipmapFormat` - 格式枚举

### 支持的格式

- **压缩格式**: DXT1, DXT3, DXT5
- **原始格式**: RGBA8888, RG88, R8
- **图片格式**: PNG, JPEG, BMP, GIF 等
- **视频格式**: MP4

---

## 使用模式

所有模块都支持两种使用模式：

### 1. 单独使用

```rust
// 一键提取壁纸
let result = paper::extract_all(ExtractInput { config });

// 一键解包 pkg
let result = pkg::unpack_pkg(UnpackPkgInput { file_path, output_base });

// 一键转换 tex
let result = tex::convert_tex(ConvertTexInput { file_path, output_path });
```

### 2. 复合流程

```rust
// 精细控制：扫描 → 检查 → 处理
let dirs = paper::list_dirs(ListDirsInput { path });
for dir in dirs.dirs {
    let check = paper::check_pkg(CheckPkgInput { folder: dir.into() });
    if check.has_pkg {
        // 走 pkg 解包流程
        let unpack_result = pkg::unpack_pkg(...);
        // 转换 tex 文件
        for file in unpack_result.extracted_files {
            if file.entry_name.ends_with(".tex") {
                tex::convert_tex(...);
            }
        }
    }
}
```
