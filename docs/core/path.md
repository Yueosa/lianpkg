# core::path 模块

路径处理与解析模块。提供统一路径解析接口、Workshop 自动探测、文件扫描。

## 核心接口

| 函数 | 签名 | 用途 |
|------|------|------|
| `resolve_path` | `(ResolvePathInput) -> CoreResult<ResolvePathOutput>` | 统一路径解析入口 |
| `detect_workshop_path` | `() -> CoreResult<PathBuf>` | 自动探测 Wallpaper Engine Workshop 路径 |
| `expand_path` | `(ExpandPathInput) -> CoreResult<ExpandPathOutput>` | 展开 `~` 为 home 目录 |
| `ensure_dir` | `(EnsureDirInput) -> CoreResult<EnsureDirOutput>` | 确保目录存在 |
| `scan_files` | `(ScanFilesInput) -> CoreResult<ScanFilesOutput>` | 递归扫描指定扩展名文件 |

## PathType 枚举

`resolve_path` 通过 `PathType` 分发到不同的路径解析逻辑：

| 变体 | 说明 | 示例结果 |
|------|------|---------|
| `ConfigDir` | 配置目录 | `~/.config/lianpkg` |
| `ConfigToml` | config.toml 路径 | `~/.config/lianpkg/config.toml` |
| `StateJson` | state.json 路径 | `~/.config/lianpkg/state.json` |
| `Workshop` | Workshop 路径（调用 `detect_workshop_path`） | `.../steamapps/workshop/content/431960` |
| `RawOutput` | 原始壁纸输出 | `~/.local/share/lianpkg/Wallpapers_Raw`（已展开） |
| `UnpackedOutput` | 解包输出 | `~/.local/share/lianpkg/Pkg_Unpacked`（已展开） |
| `SceneName { stem }` | 从 PKG 文件名提取场景名 | `"123456_scene.pkg"` → `"123456"` |
| `TexOutput { tex_path, output_base }` | TEX 转换输出目录 | `{output_base}/{stem}/tex_converted` |

所有 Linux 路径中的 `~` 在返回前已展开为实际 home 目录，`ResolvePathOutput.path` 是可直接使用的绝对路径。

## Workshop 探测流程

`detect_workshop_path()` 遍历所有 Steam library folder，找到 `steamapps/workshop/content/431960` 存在的那个。

```
1. 收集 Steam 候选基路径
   ├─ Linux: 检查 4 个候选位置（原生/符号链接/Flatpak/Snap）
   └─ Windows: 从注册表 HKCU\Software\Valve\Steam 读取 + 默认路径

2. 对每个基路径：
   ├─ 将自身加入候选列表
   └─ 解析 steamapps/libraryfolders.vdf → 提取所有 library path

3. 去重后逐个检查 {library}/steamapps/workshop/content/431960/ 是否存在

4. 返回第一个命中的路径；全部未命中 → CoreError::NotFound
```

### Linux Steam 候选路径

| 路径 | 来源 |
|------|------|
| `~/.local/share/Steam` | 原生安装 |
| `~/.steam/steam` | 常见符号链接 |
| `~/.var/app/com.valvesoftware.Steam/data/Steam` | Flatpak |
| `~/snap/steam/common/.steam/steam` | Snap |

### VDF 解析

解析 `libraryfolders.vdf` 提取所有 `"path"` 行的值。处理 `\\` → `\` 转义（Windows VDF）。仅返回 `exists()` 的路径。

## 文件结构

| 文件 | 内容 |
|------|------|
| `mod.rs` | 模块声明、导出、兼容层函数（过渡用） |
| `resolve.rs` | `resolve_path`、`detect_workshop_path`、PathType、VDF/Steam 探测 |
| `utl.rs` | `ensure_dir`、`expand_path` |
| `scan.rs` | `scan_files`（递归目录遍历 + 扩展名过滤） |
| `types.rs` | Input/Output 结构体定义 |

## 兼容层

`mod.rs` 中保留了一组 `_compat` / `default_` 函数供 `api::native` 和 `cli` 过渡使用。这些函数将在 Step 7/8（api/cli 重构）完成后删除。
