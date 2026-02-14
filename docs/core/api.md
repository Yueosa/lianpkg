# api::native — 原生 API 层

> Step 7 重构文档 — 精简为 6 个核心模块

## 概述

`api::native` 是 CLI 和未来 Flutter GUI (通过 `api::ffi`) 调用的统一接口层。  
它封装了 `core` 模块的底层操作，提供类型安全、可组合的高级 API。

### 设计原则

1. **CoreResult<T> 统一错误处理** — 所有接口返回 `CoreResult<T>`，取代旧版 `{success, error}` 模式
2. **AppContext 集中初始化** — 一次 `init()` 完成配置/状态文件的创建与加载
3. **调用者负责最终配置** — API 层不做 override 合并，CLI 在调用前组装好参数
4. **扫描与复制解耦** — `scan()` 返回壁纸列表，`copy_wallpapers()` 接受预扫描结果，避免重复扫描

## 模块结构

```
api/native/
├── context.rs    # 应用上下文与配置管理
├── scan.rs       # 壁纸扫描与复制
├── auto.rs       # 自动批处理流水线
├── pkg.rs        # PKG 解包与预览
├── tex.rs        # TEX 转换与预览
├── util.rs       # 内部工具函数 (pub(crate))
└── mod.rs        # 模块声明与 re-export
```

## context — 应用上下文

### 核心类型

| 类型 | 描述 |
|------|------|
| `AppContext` | 运行时上下文（config + 文件路径） |
| `RuntimeConfig` | 从 config.toml 解析的运行时配置 |
| `PipelineConfig` | 流水线相关配置（incremental, auto_unpack 等） |

### 接口

```rust
// 初始化（创建配置文件 + 加载配置）
fn init(config_dir: Option<PathBuf>) -> CoreResult<AppContext>

// 配置/状态操作
fn load_config(config_path: &Path) -> CoreResult<RuntimeConfig>
fn load_state(state_path: &Path) -> CoreResult<StateData>
fn load_state_or_default(state_path: &Path) -> StateData
fn save_state(state_path: &Path, state: &StateData) -> CoreResult<()>

// 状态辅助
fn is_wallpaper_processed(state: &StateData, id: &str) -> bool
fn add_processed_wallpaper(state: &mut StateData, id: String, ...)
fn touch_last_run(state: &mut StateData)
```

### 用法

```rust
let ctx = context::init(None)?;
println!("Workshop: {}", ctx.config.workshop_path.display());
```

## scan — 壁纸扫描与复制

### 核心类型

| 类型 | 描述 |
|------|------|
| `ScanOutput` | 扫描结果（wallpapers + stats） |
| `WallpaperInfo` | 单个壁纸信息（含 `is_processed` 增量标记） |
| `CopyOutput` | 复制结果（results + stats） |
| `CopyResult` | 单个壁纸复制结果 |
| `CopyResultType` | Raw / Pkg / Skipped |

### 接口

```rust
// 使用 AppContext 扫描（自动加载状态标记 is_processed）
fn scan(ctx: &AppContext) -> CoreResult<ScanOutput>

// 扫描指定目录（可选传入 state 做增量标记）
fn scan_workshop(path: &Path, state: Option<&StateData>) -> CoreResult<ScanOutput>

// 复制预扫描的壁纸（解耦扫描与复制，避免重复扫描）
fn copy_wallpapers(
    wallpapers: &[WallpaperInfo],
    raw_output_path: &Path,
    enable_raw: bool,
) -> CoreResult<CopyOutput>

// 便捷版：自动扫描 + 按 ID 过滤 + 复制
fn scan_and_copy(
    workshop_path: &Path,
    filter_ids: Option<&[String]>,
    raw_output_path: &Path,
    enable_raw: bool,
) -> CoreResult<CopyOutput>

// 获取单个壁纸详情
fn get_wallpaper_detail(workshop_path: &Path, id: &str) -> Option<WallpaperInfo>
```

### 关键改进：消除双重扫描

旧流水线中 `copy_wallpapers()` 内部会再次调用 `scan()`，导致 Workshop 被扫描两次。  
新设计中 `copy_wallpapers()` 接受 `&[WallpaperInfo]`，由调用者传入预扫描结果。

## auto — 自动批处理流水线

### 核心类型

| 类型 | 描述 |
|------|------|
| `AutoOptions<'a>` | 流水线选项（wallpaper_ids, progress callback） |
| `AutoOutput` | 流水线结果（copy/pkg/tex 各阶段结果 + 统计） |
| `AutoProgress` | 进度回调数据 |
| `AutoStage` | Init → Scanning → Copying → Unpacking → Converting → Cleanup → Done |
| `PipelineStats` | 统计（wallpapers_processed, wallpapers_skipped, pkgs_unpacked, texs_converted, elapsed_ms） |
| `DiskEstimateOutput` | 磁盘空间预估 |

### 接口

```rust
// 执行完整流水线：scan → copy → unpack → convert → cleanup
fn run_auto(ctx: &AppContext, opts: AutoOptions) -> CoreResult<AutoOutput>

// 磁盘预估
fn estimate_disk_usage(config: &RuntimeConfig) -> DiskEstimateOutput

// 专项执行
fn run_pkg_only(sources: &[PkgSource], output: &Path) -> CoreResult<UnpackOutput>
fn run_tex_only(unpacked_path: &Path, output_path: &Path) -> CoreResult<ConvertOutput>
```

### 用法

```rust
let ctx = context::init(None)?;

// 覆盖 CLI 参数
ctx.config.pipeline.incremental = true;

let result = auto::run_auto(&ctx, AutoOptions {
    wallpaper_ids: None,
    progress: Some(&|p| println!("[{:?}] {}%", p.stage, p.progress)),
})?;
```

### 关键改进：消除 PipelineOverrides

旧版 `RunPipelineInput` 包含大量 `Option<PathBuf>` override 字段，API 内部需要二次 merge。  
新设计中 CLI 在调用前直接修改 `ctx.config`，API 不再做覆盖逻辑。

## pkg — PKG 解包与预览

### 核心类型

| 类型 | 描述 |
|------|------|
| `PkgSource` | 壁纸 ID + PKG 路径列表 |
| `UnpackOutput` | 批量解包结果（results + stats） |
| `UnpackResult` | 单个 PKG 解包结果 |
| `UnpackedFile` | 解包后的文件信息 |
| `PkgPreview` | PKG 预览信息（不解包） |

### 接口

```rust
fn unpack_all(sources: &[PkgSource], output: &Path) -> CoreResult<UnpackOutput>
fn unpack_single(pkg_path: &Path, output_base: &Path) -> CoreResult<UnpackResult>
fn preview_pkg(pkg_path: &Path) -> CoreResult<PkgPreview>
```

### 签名变化

```rust
// 旧版
fn unpack_all(input: UnpackAllInput) -> UnpackAllOutput  // {success, error} 模式

// 新版
fn unpack_all(sources: &[PkgSource], output: &Path) -> CoreResult<UnpackOutput>
```

## tex — TEX 转换与预览

### 核心类型

| 类型 | 描述 |
|------|------|
| `ConvertOutput` | 批量转换结果 |
| `ConvertResult` | 单个 TEX 转换结果 |
| `ConvertStats` | 统计（tex_processed, tex_success, tex_failed, tex_skipped, image_count, video_count） |
| `TexPreview` | TEX 文件预览信息 |

### 接口

```rust
fn convert_all(unpacked_path: &Path, output_path: &Path) -> CoreResult<ConvertOutput>
fn convert_single(tex_path: &Path, output: &Path) -> CoreResult<ConvertResult>
fn preview_tex(tex_path: &Path) -> CoreResult<TexPreview>
```

## util — 内部工具函数

> `pub(crate)` 可见性，仅供 `auto.rs` 等模块内部使用

| 函数 | 描述 |
|------|------|
| `filter_wallpapers()` | 按 ID + 增量状态筛选壁纸 |
| `copy_metadata_to_tex_converted()` | 复制元数据到 TEX 转换输出目录 |
| `clean_unpacked_dir()` | 清理解包中间产物 |
| `find_tex_files()` | 递归查找 TEX 文件 |
| `determine_tex_output_path()` | 计算 TEX 输出路径（保持目录结构） |

## 与旧版对照

| 旧文件 | 行数 | 新文件 | 行数 | 变化 |
|--------|------|--------|------|------|
| cfg.rs | ~180 | context.rs | ~255 | 重写为 AppContext 模式 |
| paper.rs | ~360 | scan.rs | ~290 | 扫描/复制解耦，增加 is_processed |
| pipeline.rs | ~883 | auto.rs + util.rs | ~540 | 拆分流水线 + 工具函数 |
| pkg.rs | ~260 | pkg.rs | ~270 | CoreResult 替换 success/error |
| tex.rs | ~220 | tex.rs | ~240 | CoreResult 替换 success/error |
| types.rs | ~80 | (删除) | 0 | 未使用 |
| **总计** | **~1983** | **总计** | **~1595** | **减少 ~20%** |

## CLI 适配

所有 CLI handler 已适配新 API：

| Handler | 主要变化 |
|---------|---------|
| wallpaper.rs | `context::init()` + `scan::scan_and_copy()` |
| auto.rs | `apply_overrides()` + `auto::run_auto()` + `AutoProgress` 回调 |
| pkg.rs | `pkg::unpack_all(&sources, &path)` 直接传参 |
| tex.rs | `tex::convert_all(&path, output.as_deref())` 直接传参 |
| config.rs | `context::init()` + `cfg::update_config_toml(key, value)` |
| status.rs | `context::load_state_or_default()` + `context::save_state()` |
