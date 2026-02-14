# core::cfg 模块

> 配置文件 (`config.toml`) 与状态文件 (`state.json`) 的底层 CRUD 操作。  
> 不含业务逻辑，仅负责读/写/创建/删除文件本身。

## 文件结构

```
src/core/cfg/
├── mod.rs       # 模块声明与导出
├── structs.rs   # 全部结构体定义
├── utl.rs       # 工具函数、默认模板
├── config.rs    # config.toml CRUD（4 个接口）
├── state.rs     # state.json CRUD（4 个接口）
└── clear.rs     # 目录清理（1 个接口）
```

## 9 个接口

| 函数 | 说明 |
|------|------|
| `create_config_toml` | 创建 config.toml（已存在时跳过） |
| `read_config_toml` | 读取 config.toml 全文 |
| `update_config_toml` | 按 key 更新 config.toml 中的值 |
| `delete_config_toml` | 删除 config.toml |
| `create_state_json` | 创建 state.json（已存在时跳过） |
| `read_state_json` | 读取 state.json 全文 |
| `write_state_json` | 覆写 state.json |
| `delete_state_json` | 删除 state.json |
| `clear_lianpkg` | 递归清理指定目录 |

以上接口均返回 `CoreResult<XxxOutput>`, 错误通过 `CoreError` 传播。

---

## StateData 结构（v2 — HashMap）

```rust
pub struct StateData {
    pub processed: HashMap<String, ProcessedEntry>,  // wallpaper_id → entry
    pub last_run:  Option<String>,                   // ISO 8601, e.g. "2025-01-15T08:30:00+00:00"
}

pub struct ProcessedEntry {
    pub title:        Option<String>,
    pub process_type: ProcessType,
    pub processed_at: String,          // ISO 8601
    pub output_path:  Option<String>,
}

pub enum ProcessType { Raw, Pkg, PkgTex, Skipped }
```

### 设计决策

| 旧版 (v1) | 新版 (v2) | 理由 |
|-----------|-----------|------|
| `Vec<ProcessedWallpaper>` | `HashMap<String, ProcessedEntry>` | O(1) 查重 + 天然去重 (upsert) |
| `WallpaperProcessType` | `ProcessType` | 简化命名 |
| `StateStatistics` (累加器) | 已删除 | 所有统计可从 HashMap 实时计算 |
| `last_run: Option<u64>` | `last_run: Option<String>` | 使用 ISO 8601 字符串，人类可读 |
| `processed_at: u64` | `processed_at: String` | 同上 |

### 序列化格式 (state.json)

```json
{
  "processed": {
    "12345678": {
      "title": "Firewatch",
      "process_type": "Raw",
      "processed_at": "2025-01-15T08:30:00+00:00",
      "output_path": "/home/user/Wallpapers_Raw/12345678"
    },
    "87654321": {
      "title": "Night Sky",
      "process_type": "Pkg",
      "processed_at": "2025-01-15T08:31:12+00:00",
      "output_path": null
    }
  },
  "last_run": "2025-01-15T08:31:12+00:00"
}
```

### 向后兼容

旧版 Vec 格式的 `state.json` 无法被新版解析。  
`load_state()` 会返回解析失败，调用方 fallback 到 `StateData::default()`（空 HashMap）。  
最坏情况：所有壁纸被重新处理一次。对于增量模式用户，首次运行后即恢复正常。

---

## 默认模板

### config.toml

由 `utl::default_config_template()` 生成，包含：
- `[wallpaper]`: workshop_path（自动探测）、raw_output_path、enable_raw_output
- `[unpack]`: unpacked_output_path、clean_unpacked
- `[tex]`: converted_output_path（可选）
- `[pipeline]`: incremental、auto_unpack_pkg、auto_convert_tex

### state.json

默认模板: `{"processed":{}}`

---

## 工具函数

| 函数 | 说明 |
|------|------|
| `utl::default_config_template()` | 生成带注释的 config.toml 默认内容 |
| `utl::default_state_template()` | 返回 `{"processed":{}}` |
| `utl::ensure_dir(path)` | 递归创建目录 |

## Clear 接口

```rust
pub struct ClearInput  { pub dir_path: PathBuf }
pub struct ClearOutput { pub cleared: bool, pub deleted_items: Vec<DeletedItem> }
pub struct DeletedItem { pub path: PathBuf, pub item_type: ItemType }
pub enum ItemType { File, Directory }
```

遍历目标目录，逐项删除并记录，返回删除清单。
