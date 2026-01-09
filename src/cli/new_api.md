# CLI 接口变更记录

本文档记录 core 模块重构后的 API 变更，供 CLI 模块后续迁移参考。

## 变更日期：2026-01-09

---

## 1. 错误处理

### 新增
- `CoreError` 枚举 (`src/core/error.rs`)
  - `Io { message, path }` - IO 错误
  - `Parse { message, path }` - 解析错误
  - `Validation { message }` - 验证错误
  - `NotFound { message, path }` - 资源未找到
  - `Unsupported { message }` - 不支持的操作

- `CoreResult<T>` 类型别名 = `Result<T, CoreError>`

### 迁移指南
所有 core 函数现在返回 `CoreResult<T>` 而非 `Result<T, Box<dyn Error>>` 或直接 panic。

---

## 2. path 模块重构

### 旧接口 (17 个) → 新接口 (4 个)

| 旧接口                           | 新接口                                        | 说明                |
| -------------------------------- | --------------------------------------------- | ------------------- |
| `default_config_dir()`           | `resolve_path(PathType::ConfigDir, ...)`      | 配置目录            |
| `default_config_path()`          | `resolve_path(PathType::ConfigToml, ...)`     | config.toml 路径    |
| `default_state_path()`           | `resolve_path(PathType::StateJson, ...)`      | state.json 路径     |
| `default_workshop_path()`        | `resolve_path(PathType::Workshop, ...)`       | Steam Workshop 路径 |
| `default_raw_output_path()`      | `resolve_path(PathType::RawOutput, ...)`      | 原始输出目录        |
| `default_pkg_temp_path()`        | `resolve_path(PathType::PkgTemp, ...)`        | pkg 临时目录        |
| `default_unpacked_output_path()` | `resolve_path(PathType::UnpackedOutput, ...)` | 解包输出目录        |
| `pkg_temp_dest()`                | `resolve_path(PathType::PkgTempDest, ...)`    | pkg 临时目标路径    |
| `scene_name_from_pkg_stem()`     | `resolve_path(PathType::SceneName, ...)`      | 场景名称提取        |
| `resolve_tex_output_dir()`       | `resolve_path(PathType::TexOutput, ...)`      | tex 输出目录        |
| `expand_path()`                  | `expand_path(ExpandPathInput)`                | 路径展开            |
| `ensure_dir()`                   | `ensure_dir(EnsureDirInput)`                  | 确保目录存在        |
| `scan_files()`                   | `scan_files(ScanFilesInput)`                  | 扫描文件            |

### 新的 PathType 枚举
```rust
pub enum PathType {
    ConfigDir,      // 配置目录 (~/.config/lianpkg 或 %APPDATA%\lianpkg)
    ConfigToml,     // config.toml 路径
    StateJson,      // state.json 路径
    Workshop,       // Steam Workshop 路径
    RawOutput,      // 原始输出目录
    PkgTemp,        // pkg 临时目录
    UnpackedOutput, // 解包输出目录
    PkgTempDest,    // pkg 临时目标路径 (需要 pkg_path 参数)
    SceneName,      // 场景名称 (需要 pkg_path 参数)
    TexOutput,      // tex 输出目录 (需要 tex_path + output_base 参数)
}
```

### 新的统一调用方式
```rust
use crate::core::path::{resolve_path, ResolvePathInput, PathType};

// 获取配置目录
let result = resolve_path(ResolvePathInput {
    path_type: PathType::ConfigDir,
    ..Default::default()
})?;

// 获取 pkg 临时目标路径
let result = resolve_path(ResolvePathInput {
    path_type: PathType::PkgTempDest,
    pkg_path: Some("/path/to/file.pkg".to_string()),
    ..Default::default()
})?;
```

### 兼容层
`src/core/path/mod.rs` 提供了兼容函数，CLI 可继续使用旧接口名：
- `expand_path_compat()`, `ensure_dir_compat()`
- `default_config_dir()`, `default_workshop_path()` 等
- `resolve_tex_output_dir_compat()`

---

## 3. cfg 模块

### 接口数量：9 个（无变化）

| 接口                 | 说明              |
| -------------------- | ----------------- |
| `create_config_toml` | 创建 config.toml  |
| `read_config_toml`   | 读取 config.toml  |
| `update_config_toml` | 更新 config.toml  |
| `delete_config_toml` | 删除 config.toml  |
| `create_state_json`  | 创建 state.json   |
| `read_state_json`    | 读取 state.json   |
| `write_state_json`   | 写入 state.json   |
| `delete_state_json`  | 删除 state.json   |
| `clear_lianpkg`      | 清理 lianpkg 目录 |

### 变更
- 所有函数返回值从 `Result<T, Box<dyn Error>>` 改为 `CoreResult<T>`

---

## 4. paper 模块

### 接口数量：6 个（无变化）

| 接口             | 说明                     |
| ---------------- | ------------------------ |
| `list_dirs`      | 列出壁纸目录             |
| `read_meta`      | 读取 project.json 元数据 |
| `check_pkg`      | 检查是否有 pkg 文件      |
| `estimate`       | 估算处理结果             |
| `process_folder` | 处理单个文件夹           |
| `extract_all`    | 一键提取所有壁纸         |

### 变更
- 所有函数返回值改为 `CoreResult<T>`

---

## 5. pkg 模块

### 接口数量：3 个（无变化）

| 接口           | 说明              |
| -------------- | ----------------- |
| `parse_pkg`    | 解析 pkg 文件结构 |
| `unpack_pkg`   | 一键解包整个 pkg  |
| `unpack_entry` | 解包单个条目      |

### 变更
- 所有函数返回值改为 `CoreResult<T>`

---

## 6. tex 模块

### 接口数量：2 个（无变化）

| 接口          | 说明                 |
| ------------- | -------------------- |
| `parse_tex`   | 解析 tex 文件头信息  |
| `convert_tex` | 转换 tex 为图片/视频 |

### 变更
- 所有函数返回值改为 `CoreResult<T>`

---

## CLI 迁移检查清单

- [ ] 更新错误处理，使用 `CoreError` 匹配
- [ ] path 模块：可继续用兼容层，或迁移到 `resolve_path()`
- [ ] cfg 模块：更新 `?` 操作符的错误类型
- [ ] paper 模块：更新错误处理
- [ ] pkg 模块：更新错误处理
- [ ] tex 模块：更新错误处理
