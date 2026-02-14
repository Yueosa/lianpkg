# LianPkg GUI — Flutter 图形界面

> 基于 Flutter + Riverpod + Rust FFI 的桌面客户端，提供壁纸浏览、流水线执行、配置管理等可视化操作。

---

## 技术栈

| 组件 | 版本 | 用途 |
|------|------|------|
| Flutter | 3.42.0 (master) | UI 框架 |
| Dart | ≥ 3.12.0 | 编程语言 |
| Material 3 | — | 设计系统 |
| flutter_riverpod | 2.6.1 | 状态管理 |
| ffi | 2.2.0 | Dart ↔ Rust FFI 桥接 |
| path | 1.9.1 | 路径处理 |

---

## 项目结构

```
gui/lib/
├── main.dart                  # 入口，主题定义（粉蓝白配色）
│
├── services/                  # 服务层
│   ├── ffi_bridge.dart        # FFI 通信桥（JSON 序列化/反序列化）
│   └── lianpkg_service.dart   # 业务 API 封装（类型安全的 Dart 接口）
│
├── models/                    # 数据模型（对应 Rust 结构体的 Dart 映射）
│   ├── config.dart            # LianpkgConfig, RuntimeConfig
│   ├── wallpaper.dart         # ScanResult, WallpaperInfo, StatusInfo
│   ├── pipeline.dart          # AutoOutput, PipelineStats, ProgressSnapshot
│   ├── state.dart             # StateData, ProcessedEntry
│   ├── pkg_preview.dart       # PkgPreview
│   └── tex_preview.dart       # TexPreview
│
├── providers/                 # Riverpod Provider（全局状态管理）
│   └── providers.dart         # 所有 Provider 定义
│
├── widgets/                   # 公共组件
│   └── app_shell.dart         # NavigationRail + 页面容器 + 导航刷新逻辑
│
├── pages/                     # 页面
│   ├── dashboard_page.dart    # 总览仪表盘
│   ├── browser_page.dart      # 壁纸浏览
│   ├── pipeline_page.dart     # 自动流水线
│   ├── settings_page.dart     # 配置管理
│   ├── detail_page.dart       # 单壁纸详情
│   └── state_page.dart        # 处理状态
│
└── utils/                     # 工具函数
    └── open_folder.dart       # xdg-open / explorer 打开文件夹
```

---

## 架构

```
┌──────────────────────────────────────────────────────┐
│                   Flutter UI (pages/)                 │
│   Dashboard  Browser  Pipeline  Settings  Detail     │
└──────────────────────┬───────────────────────────────┘
                       │ ref.watch / ref.read
┌──────────────────────▼───────────────────────────────┐
│              Riverpod Providers (providers.dart)      │
│   initProvider → configProvider / scanResultProvider  │
│   statusProvider / stateProvider / navigationIndex    │
└──────────────────────┬───────────────────────────────┘
                       │ service.xxx()
┌──────────────────────▼───────────────────────────────┐
│           LianpkgService (lianpkg_service.dart)      │
│   init / scan / runAuto / getConfig / setConfig ...  │
└──────────────────────┬───────────────────────────────┘
                       │ _ffi.callAsync(action, params)
┌──────────────────────▼───────────────────────────────┐
│              FfiBridge (ffi_bridge.dart)              │
│   callSync(action, params) → Map<String, dynamic>    │
│   callAsync(action, params) → Future<Map>            │
└──────────────────────┬───────────────────────────────┘
                       │ DynamicLibrary.open → lianpkg_call
┌──────────────────────▼───────────────────────────────┐
│              Rust FFI (liblianpkg.so / lianpkg.dll)  │
│   lianpkg_call(json) → json                         │
│   lianpkg_free_string(ptr)                           │
└──────────────────────────────────────────────────────┘
```

---

## 页面说明

### 总览仪表盘 (Dashboard)

- 壁纸统计卡片：总数 / 已处理 / 待处理 / PKG / Raw
- 磁盘占用卡片：Raw 输出 / 解包产物 / 转换产物（点击可打开目录）
- 最近处理记录

### 壁纸浏览 (Browser)

- 缩略图网格展示所有壁纸（预览图 + 标题 + 类型标签）
- 搜索筛选：按标题关键词
- 点击进入单壁纸详情页
- 壁纸文件夹不存在时自动保护提示

### 自动流水线 (Pipeline)

- 可视化流程图：扫描 → 复制 Raw → 解包 PKG → 转换 TEX → 完成（5 步）
- 开关配置：增量处理、复制 Raw、解包 PKG、转换 TEX、清理中间产物
- 实时进度显示：当前阶段高亮 + 旋转动画
- 结果面板：统计数据 + 耗时 + 详细统计

### 配置管理 (Settings)

- 路径配置：Workshop 路径、Raw 输出、解包输出、转换输出（带文件夹打开按钮）
- 开关配置：启用 Raw 输出、清理中间产物
- 流水线配置：增量处理、自动解包、自动转换
- 重置按钮

### 单壁纸详情 (Detail)

- 大图预览
- 元数据显示：标题、类型、Workshop ID、标签
- PKG 预览：列出 PKG 内文件
- TEX 预览：格式、尺寸、压缩信息

---

## 状态管理

所有全局状态通过 Riverpod Provider 管理：

| Provider | 类型 | 说明 |
|----------|------|------|
| `lianpkgServiceProvider` | `Provider` | LianpkgService 单例 |
| `initProvider` | `FutureProvider` | 初始化 Rust 上下文（其他 Provider 依赖此项） |
| `configProvider` | `FutureProvider` | 当前配置（每次从 Rust 端读取） |
| `scanResultProvider` | `FutureProvider` | Workshop 扫描结果 |
| `statusProvider` | `FutureProvider` | 综合状态统计 |
| `stateProvider` | `FutureProvider` | 处理状态数据 |
| `navigationIndexProvider` | `StateProvider` | 当前导航页索引 |

### 导航刷新机制

切换标签页时自动 `ref.invalidate()` 对应 Provider：

| 标签 | 刷新的 Provider |
|------|----------------|
| 总览 | `statusProvider` + `stateProvider` |
| 浏览 | `scanResultProvider` |
| 流水线 | `configProvider` |

---

## FFI 通信

GUI 通过 `FfiBridge` 与 Rust 核心通信，详见 [FFI 集成文档](ffi.md)。

核心要点：

- **共享库搜索顺序**：可执行文件目录 → `lib/` 子目录 → 开发路径 → 系统 PATH
- **Linux**: `liblianpkg.so`（编译进 `lib/libapp.so`，不需要额外 .so 文件）
- **Windows**: `lianpkg.dll`（放在可执行文件同目录或 `lib/` 下）

---

## 主题

粉蓝白配色方案，支持亮色/暗色跟随系统：

| 用途 | 颜色 |
|------|------|
| Primary Seed | `#E8839B`（柔粉） |
| Secondary Seed | `#7EB8D8`（浅蓝） |
| Light Surface | `#FCF8FA` |
| Dark Surface | `#1A1520` |

---

## 构建

### Linux

```bash
cd gui
flutter build linux --release
```

产物位于 `gui/build/linux/x64/release/bundle/`，包含：

```
bundle/
├── lianpkg-gui            # 可执行文件
├── lib/
│   ├── libflutter_linux_gtk.so
│   └── libapp.so          # 应用逻辑 + Rust FFI
└── data/
    ├── icudtl.dat
    └── flutter_assets/
```

### Windows

Windows GUI 需在 Windows 环境下构建（Flutter Desktop 不支持交叉编译），推荐使用 GitHub Actions：

```bash
# 在 Windows 上
cargo build --release --lib        # 生成 lianpkg.dll
cd gui
flutter build windows --release    # 生成 Windows GUI
# 将 lianpkg.dll 复制到 bundle 的 Release 目录下
```

---

## 分发

### Linux

- **AUR**: `lianpkg-gui-bin`（安装到 `/opt/lianpkg-gui/`，创建桌面快捷方式）
- **tar.gz**: 解压即用，保持 `lianpkg-gui` + `lib/` + `data/` 相对位置

### Windows

- **zip**: 解压后运行 `lianpkg-gui.exe`（需确保 `lianpkg.dll` 在同目录或 `lib/` 下）

---

*最后更新于 2026-02-15，对应 LianPkg GUI v2.0.1+1。*
