# path 模块

## 概述

- **模块职责**: 提供路径处理工具，包括路径展开、目录创建、统一路径解析和文件扫描
- **使用场景**: 被 cfg、paper、pkg、tex 模块依赖，提供跨平台路径支持
- **依赖关系**: 依赖 `error` 模块的 `CoreResult`

## 接口列表

| 函数           | 输入               | 输出                | 说明                      |
| -------------- | ------------------ | ------------------- | ------------------------- |
| `expand_path`  | `ExpandPathInput`  | `ExpandPathOutput`  | 展开 `~` 为用户主目录     |
| `ensure_dir`   | `EnsureDirInput`   | `EnsureDirOutput`   | 确保目录存在              |
| `resolve_path` | `ResolvePathInput` | `ResolvePathOutput` | 统一路径解析（10 种类型） |
| `scan_files`   | `ScanFilesInput`   | `ScanFilesOutput`   | 扫描目标文件              |

## 数据结构

### PathType 枚举

统一路径类型，替代原有 17 个独立函数：

```rust
pub enum PathType {
    /// 配置目录 (~/.config/lianpkg 或 %APPDATA%\lianpkg)
    ConfigDir,
    /// config.toml 文件路径
    ConfigToml,
    /// state.json 文件路径
    StateJson,
    /// Steam Workshop 路径
    Workshop,
    /// 原始壁纸输出路径
    RawOutput,
    /// PKG 临时路径
    PkgTemp,
    /// 解包输出路径
    UnpackedOutput,
    /// PKG 临时目标名 (需要 dir_name + file_name)
    PkgTempDest { dir_name: String, file_name: String },
    /// 从 PKG stem 提取场景名 (需要 stem)
    SceneName { stem: String },
    /// TEX 输出目录 (需要 tex_path + output_base)
    TexOutput { tex_path: PathBuf, output_base: PathBuf },
}
```

### Input 结构体

#### ExpandPathInput
```rust
pub struct ExpandPathInput {
    /// 待展开的路径字符串（可能包含 ~）
    pub path: String,
}
```

#### EnsureDirInput
```rust
pub struct EnsureDirInput {
    /// 目标目录路径
    pub path: PathBuf,
}
```

#### ResolvePathInput
```rust
pub struct ResolvePathInput {
    /// 路径类型
    pub path_type: PathType,
}
```

#### ScanFilesInput
```rust
pub struct ScanFilesInput {
    /// 搜索路径（文件或目录）
    pub path: PathBuf,
    /// 文件扩展名过滤（可选，默认 ["pkg", "tex"]）
    pub extensions: Option<Vec<String>>,
}
```

### Output 结构体

#### ExpandPathOutput
```rust
pub struct ExpandPathOutput {
    /// 展开后的完整路径
    pub path: PathBuf,
}
```

#### EnsureDirOutput
```rust
pub struct EnsureDirOutput {
    /// 目录路径
    pub path: PathBuf,
    /// 是否新创建（false 表示已存在）
    pub created: bool,
}
```

#### ResolvePathOutput
```rust
pub struct ResolvePathOutput {
    /// 解析后的路径
    pub path: PathBuf,
    /// 路径字符串形式
    pub path_str: String,
}
```

#### ScanFilesOutput
```rust
pub struct ScanFilesOutput {
    /// 目标文件列表
    pub files: Vec<PathBuf>,
}
```

## 接口详解

### `expand_path`

- **签名**: `fn expand_path(input: ExpandPathInput) -> CoreResult<ExpandPathOutput>`
- **功能**: 将路径中的 `~` 展开为用户主目录

| 输入             | 输出                             |
| ---------------- | -------------------------------- |
| `~/config`       | `/home/user/config` (Linux)      |
| `~\config`       | `C:\Users\user\config` (Windows) |
| `/absolute/path` | `/absolute/path` (不变)          |

**错误**:
- `CoreError::Io`: 无法获取用户主目录

### `ensure_dir`

- **签名**: `fn ensure_dir(input: EnsureDirInput) -> CoreResult<EnsureDirOutput>`
- **功能**: 确保目录存在，不存在则递归创建

**处理流程**:
1. 检查目录是否存在
2. 不存在则调用 `fs::create_dir_all`
3. 返回 `created` 标志指示是否新建

**错误**:
- `CoreError::Io`: 创建目录失败（权限不足等）

### `resolve_path`

- **签名**: `fn resolve_path(input: ResolvePathInput) -> CoreResult<ResolvePathOutput>`
- **功能**: 根据 PathType 解析对应路径

#### PathType 详解

| 类型             | Linux 默认值                    | Windows 默认值                           |
| ---------------- | ------------------------------- | ---------------------------------------- |
| `ConfigDir`      | `~/.config/lianpkg`             | `%APPDATA%\lianpkg`                      |
| `ConfigToml`     | `~/.config/lianpkg/config.toml` | `%APPDATA%\lianpkg\config.toml`          |
| `StateJson`      | `~/.config/lianpkg/state.json`  | `%APPDATA%\lianpkg\state.json`           |
| `Workshop`       | Steam Workshop 自动检测         | Steam Workshop 自动检测                  |
| `RawOutput`      | `~/Pictures/WallpaperEngine`    | `%USERPROFILE%\Pictures\WallpaperEngine` |
| `PkgTemp`        | `/tmp/lianpkg_temp`             | `%TEMP%\lianpkg_temp`                    |
| `UnpackedOutput` | `~/lianpkg_unpacked`            | `%USERPROFILE%\lianpkg_unpacked`         |

#### 带参数的 PathType

##### PkgTempDest
```rust
PathType::PkgTempDest { dir_name: "scene1".into(), file_name: "scene.pkg".into() }
// 输出: /tmp/lianpkg_temp/scene1/scene.pkg
```

##### SceneName
```rust
PathType::SceneName { stem: "scene1_scene".into() }
// 输出: scene1 (去掉 _scene 后缀)
```

##### TexOutput
```rust
PathType::TexOutput {
    tex_path: "/path/to/texture.tex".into(),
    output_base: "/output".into()
}
// 输出: /output/texture/
```

**错误**:
- `CoreError::Io`: 路径解析失败

### `scan_files`

- **签名**: `fn scan_files(input: ScanFilesInput) -> CoreResult<ScanFilesOutput>`
- **功能**: 扫描指定路径下的目标文件

**处理流程**:
1. 判断输入是文件还是目录
2. 如果是文件，检查扩展名是否匹配
3. 如果是目录，递归扫描所有匹配文件

**默认扩展名**: `["pkg", "tex"]`

**错误**:
- `CoreError::NotFound`: 路径不存在
- `CoreError::Io`: 读取目录失败

## 兼容层

为简化迁移，`mod.rs` 提供了兼容函数：

```rust
// 旧风格调用
pub fn expand_path_compat(path: &str) -> CoreResult<PathBuf>;
pub fn ensure_dir_compat(path: &Path) -> CoreResult<bool>;

// 快捷路径获取
pub fn default_config_dir() -> CoreResult<PathBuf>;
pub fn default_workshop_path() -> CoreResult<String>;
pub fn default_raw_output_path() -> CoreResult<String>;
pub fn default_pkg_temp_path() -> CoreResult<String>;
pub fn default_unpacked_output_path() -> CoreResult<String>;
pub fn pkg_temp_dest(dir_name: &str, file_name: &str) -> CoreResult<String>;
pub fn scene_name_from_pkg_stem(stem: &str) -> String;
pub fn resolve_tex_output_dir_compat(tex_path: &Path, output_base: &Path) -> CoreResult<PathBuf>;
```

## 使用示例

### 基础用法

```rust
use crate::core::path::*;

// 展开路径
let result = expand_path(ExpandPathInput {
    path: "~/Documents".to_string(),
})?;
println!("展开后: {:?}", result.path);

// 确保目录存在
let result = ensure_dir(EnsureDirInput {
    path: "/tmp/my_app".into(),
})?;
if result.created {
    println!("目录已创建");
}

// 扫描文件
let result = scan_files(ScanFilesInput {
    path: "/path/to/wallpapers".into(),
    extensions: Some(vec!["pkg".to_string()]),
})?;
println!("找到 {} 个文件", result.files.len());
```

### 统一路径解析

```rust
use crate::core::path::{resolve_path, ResolvePathInput, PathType};

// 获取配置目录
let config_dir = resolve_path(ResolvePathInput {
    path_type: PathType::ConfigDir,
})?;

// 获取 Steam Workshop 路径
let workshop = resolve_path(ResolvePathInput {
    path_type: PathType::Workshop,
})?;

// 获取 PKG 临时目标路径
let pkg_dest = resolve_path(ResolvePathInput {
    path_type: PathType::PkgTempDest {
        dir_name: "scene_12345".to_string(),
        file_name: "scene.pkg".to_string(),
    },
})?;

// 获取 TEX 输出目录
let tex_out = resolve_path(ResolvePathInput {
    path_type: PathType::TexOutput {
        tex_path: "/input/texture.tex".into(),
        output_base: "/output".into(),
    },
})?;
```

## 跨平台支持

### Linux
- 配置目录: `~/.config/lianpkg`
- 临时目录: `/tmp/lianpkg_temp`
- Steam Workshop: 从 `~/.steam` 或 `~/.local/share/Steam` 检测

### Windows
- 配置目录: `%APPDATA%\lianpkg`
- 临时目录: `%TEMP%\lianpkg_temp`
- Steam Workshop: 从注册表或默认路径 `C:\Program Files (x86)\Steam` 检测

## 设计说明

### 为什么用 PathType 枚举统一接口?

1. **减少 API 表面积**: 17 个函数 → 1 个 `resolve_path` + 3 个工具函数
2. **一致的 Input/Output 模式**: 便于 FFI 序列化
3. **可扩展**: 新增路径类型只需添加枚举变体
4. **类型安全**: 编译期检查必需参数
