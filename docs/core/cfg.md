# cfg 模块

## 概述

- **模块职责**: 管理配置文件 (`config.toml`) 和状态文件 (`state.json`) 的 CRUD 操作
- **使用场景**: 初始化配置、读写用户设置、记录处理状态
- **依赖关系**: 依赖 `error` 模块的 `CoreResult`，依赖 `path` 模块获取默认路径

## 接口列表

| 函数                 | 输入                | 输出                 | 说明              |
| -------------------- | ------------------- | -------------------- | ----------------- |
| `create_config_toml` | `CreateConfigInput` | `CreateConfigOutput` | 创建配置文件      |
| `read_config_toml`   | `ReadConfigInput`   | `ReadConfigOutput`   | 读取配置文件      |
| `update_config_toml` | `UpdateConfigInput` | `UpdateConfigOutput` | 更新配置项        |
| `delete_config_toml` | `DeleteConfigInput` | `DeleteConfigOutput` | 删除配置文件      |
| `create_state_json`  | `CreateStateInput`  | `CreateStateOutput`  | 创建状态文件      |
| `read_state_json`    | `ReadStateInput`    | `ReadStateOutput`    | 读取状态文件      |
| `write_state_json`   | `WriteStateInput`   | `WriteStateOutput`   | 写入状态文件      |
| `delete_state_json`  | `DeleteStateInput`  | `DeleteStateOutput`  | 删除状态文件      |
| `clear_lianpkg`      | `ClearInput`        | `ClearOutput`        | 清理 lianpkg 目录 |

## 数据结构

### config.toml Input/Output

#### CreateConfigInput
```rust
pub struct CreateConfigInput {
    /// 配置文件路径
    pub path: PathBuf,
    /// 配置文件内容，None 则使用默认模板
    pub content: Option<String>,
}
```

#### CreateConfigOutput
```rust
pub struct CreateConfigOutput {
    /// 是否触发了创建操作（文件已存在时为 false）
    pub created: bool,
    /// 配置文件路径
    pub path: PathBuf,
}
```

#### ReadConfigInput / ReadConfigOutput
```rust
pub struct ReadConfigInput {
    pub path: PathBuf,
}

pub struct ReadConfigOutput {
    pub content: String,
}
```

#### UpdateConfigInput / UpdateConfigOutput
```rust
pub struct UpdateConfigInput {
    pub path: PathBuf,
    /// 项名（支持点号分隔的嵌套键，如 "wallpaper.workshop_path"）
    pub key: String,
    /// 项值
    pub value: String,
}

pub struct UpdateConfigOutput {
    pub content: String,  // 更新后的完整内容
}
```

#### DeleteConfigInput / DeleteConfigOutput
```rust
pub struct DeleteConfigInput {
    pub path: PathBuf,
}

pub struct DeleteConfigOutput {
    pub deleted: bool,
    pub path: PathBuf,
}
```

### state.json Input/Output

#### CreateStateInput / CreateStateOutput
```rust
pub struct CreateStateInput {
    pub path: PathBuf,
    pub content: Option<String>,  // None 则使用 "{}"
}

pub struct CreateStateOutput {
    pub created: bool,
    pub path: PathBuf,
}
```

#### ReadStateInput / ReadStateOutput
```rust
pub struct ReadStateInput {
    pub path: PathBuf,
}

pub struct ReadStateOutput {
    pub content: String,
}
```

#### WriteStateInput / WriteStateOutput
```rust
pub struct WriteStateInput {
    pub path: PathBuf,
    pub content: String,  // 覆写内容
}

pub struct WriteStateOutput {
    pub content: String,
}
```

#### DeleteStateInput / DeleteStateOutput
```rust
pub struct DeleteStateInput {
    pub path: PathBuf,
}

pub struct DeleteStateOutput {
    pub deleted: bool,
    pub path: PathBuf,
}
```

### Clear Input/Output

#### ClearInput / ClearOutput
```rust
pub struct ClearInput {
    pub dir_path: PathBuf,
}

pub struct ClearOutput {
    pub cleared: bool,
    pub deleted_items: Vec<DeletedItem>,
}

pub struct DeletedItem {
    pub path: PathBuf,
    pub item_type: ItemType,
}

pub enum ItemType {
    File,
    Directory,
}
```

### 运行时结构体

#### StateData
```rust
pub struct StateData {
    pub processed_wallpapers: Vec<ProcessedWallpaper>,
    pub last_run: Option<u64>,  // Unix 时间戳
    pub statistics: StateStatistics,
}
```

#### ProcessedWallpaper
```rust
pub struct ProcessedWallpaper {
    pub wallpaper_id: String,       // 文件夹名/workshop id
    pub title: Option<String>,      // 从 project.json 读取
    pub process_type: WallpaperProcessType,
    pub processed_at: u64,          // Unix 时间戳
    pub output_path: Option<String>,
}

pub enum WallpaperProcessType {
    Raw,      // 直接复制
    Pkg,      // 已解包
    PkgTex,   // 已解包并转换
    Skipped,  // 跳过
}
```

#### StateStatistics
```rust
pub struct StateStatistics {
    pub total_runs: u64,
    pub total_wallpapers: u64,
    pub total_pkgs: u64,
    pub total_texs: u64,
}
```

## 接口详解

### `create_config_toml`

- **签名**: `fn create_config_toml(input: CreateConfigInput) -> CoreResult<CreateConfigOutput>`
- **功能**: 创建配置文件，支持自定义内容或使用默认模板

**处理流程**:
1. 检查文件是否已存在
2. 如果存在，返回 `created: false`
3. 如果不存在，创建父目录，写入内容
4. 返回 `created: true`

**默认模板**:
```toml
# lianpkg 配置文件
[wallpaper]
workshop_path = ""
raw_output = ""

[pkg]
temp_path = ""
unpacked_output = ""
```

**错误**:
- `CoreError::Io`: 创建目录/写入文件失败

### `read_config_toml`

- **签名**: `fn read_config_toml(input: ReadConfigInput) -> CoreResult<ReadConfigOutput>`
- **功能**: 读取配置文件内容

**错误**:
- `CoreError::NotFound`: 文件不存在
- `CoreError::Io`: 读取失败

### `update_config_toml`

- **签名**: `fn update_config_toml(input: UpdateConfigInput) -> CoreResult<UpdateConfigOutput>`
- **功能**: 更新配置文件中的指定项

**支持的 key 格式**:
- 顶级键: `"key"`
- 嵌套键: `"section.key"`（如 `"wallpaper.workshop_path"`）

**处理流程**:
1. 读取现有配置
2. 解析为 TOML
3. 按点号分割 key，定位到目标位置
4. 更新值
5. 序列化并写回文件

**错误**:
- `CoreError::NotFound`: 文件不存在
- `CoreError::Parse`: TOML 解析失败
- `CoreError::Validation`: 无效的 key 路径
- `CoreError::Io`: 写入失败

### `delete_config_toml`

- **签名**: `fn delete_config_toml(input: DeleteConfigInput) -> CoreResult<DeleteConfigOutput>`
- **功能**: 删除配置文件

**错误**:
- `CoreError::Io`: 删除失败

### `create_state_json` / `read_state_json` / `write_state_json` / `delete_state_json`

与 config.toml 操作类似，区别：
- 默认内容为 `{}`（空 JSON 对象）
- `write_state_json` 直接覆写整个文件（不做部分更新）

### `clear_lianpkg`

- **签名**: `fn clear_lianpkg(input: ClearInput) -> CoreResult<ClearOutput>`
- **功能**: 清理指定目录下的所有内容

**处理流程**:
1. 检查目录是否存在
2. 遍历目录内容
3. 递归删除文件和子目录
4. 记录每个删除项
5. 返回删除项列表

**错误**:
- `CoreError::Io`: 删除操作失败

## 使用示例

### 创建和读取配置

```rust
use crate::core::cfg::*;
use std::path::PathBuf;

// 创建配置文件
let result = create_config_toml(CreateConfigInput {
    path: PathBuf::from("/home/user/.config/lianpkg/config.toml"),
    content: None,  // 使用默认模板
})?;

if result.created {
    println!("配置文件已创建: {:?}", result.path);
}

// 读取配置
let result = read_config_toml(ReadConfigInput {
    path: PathBuf::from("/home/user/.config/lianpkg/config.toml"),
})?;
println!("配置内容:\n{}", result.content);
```

### 更新配置项

```rust
// 更新 workshop 路径
let result = update_config_toml(UpdateConfigInput {
    path: PathBuf::from("/home/user/.config/lianpkg/config.toml"),
    key: "wallpaper.workshop_path".to_string(),
    value: "/home/user/.steam/workshop".to_string(),
})?;
```

### 管理状态文件

```rust
use crate::core::cfg::*;
use serde_json;

// 读取状态
let result = read_state_json(ReadStateInput {
    path: PathBuf::from("/home/user/.config/lianpkg/state.json"),
})?;
let state: StateData = serde_json::from_str(&result.content)?;

// 添加已处理壁纸
let mut state = state;
state.processed_wallpapers.push(ProcessedWallpaper {
    wallpaper_id: "12345".to_string(),
    title: Some("My Wallpaper".to_string()),
    process_type: WallpaperProcessType::Pkg,
    processed_at: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs(),
    output_path: Some("/output/12345".to_string()),
});

// 写回状态
let content = serde_json::to_string_pretty(&state)?;
write_state_json(WriteStateInput {
    path: PathBuf::from("/home/user/.config/lianpkg/state.json"),
    content,
})?;
```

### 清理目录

```rust
let result = clear_lianpkg(ClearInput {
    dir_path: PathBuf::from("/tmp/lianpkg_temp"),
})?;

if result.cleared {
    for item in result.deleted_items {
        println!("已删除 {:?}: {:?}", item.item_type, item.path);
    }
}
```

## 设计说明

### 为什么分离 config.toml 和 state.json?

- **config.toml**: 用户配置，人类可读可编辑
- **state.json**: 程序状态，机器读写，支持复杂嵌套结构

### 为什么 write_state_json 是全量覆写?

JSON 的部分更新比 TOML 复杂（需要处理数组增删），且状态文件通常整体读写，全量覆写更简单可靠。
