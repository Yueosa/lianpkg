## src 目录结构

```
src/
	lib.rs            # 导出 core 与 api
	main.rs           # CLI 入口
	api/
		mod.rs          # 汇总 native/ffi/types
		native.rs       # 纯 Rust API（直接调用核心逻辑）
		ffi.rs          # C 接口导出（接受 JSON 配置）
		types.rs        # API 通用返回类型
	cli/
		mod.rs          # clap 命令行定义与入口
		handlers.rs     # 针对每个子命令的执行封装
		logger.rs       # 轻量日志输出（可调试模式）
	core/
		mod.rs          # 核心模块聚合
		config/mod.rs   # 配置结构与合并加载
		path/mod.rs     # 路径工具：默认路径、文件搜寻等
		paper/mod.rs    # 壁纸收集（拷贝 raw / 复制 pkg）
		pkg/mod.rs      # .pkg 解包器
		tex/
			mod.rs        # .tex 处理入口
			reader.rs     # TEX 格式解析
			converter.rs  # TEX -> 图片/视频 转换
			structs.rs    # TEX 数据结构
```

## 配置结构（Config JSON/TOML）

字段与类型（对应 core::config::Config）：
- wallpaper: { workshop_path: String, raw_output_path: String, pkg_temp_path: String, enable_raw_output: bool }
- unpack: { unpacked_output_path: String, clean_pkg_temp: bool, clean_unpacked: bool }
- tex: { converted_output_path: Option<String> }

默认路径（按平台自动展开）来自 core::path::default_*，可用 `~` 开头路径自动展开。

## API 库（Rust 调用）

模块 api::native（直接返回 Result）：
- run_wallpaper(config: &Config) -> Result<WallpaperStats, String>
	- 作用：扫描 workshop，复制 raw 壁纸或 .pkg 到临时目录。
- run_pkg(config: &Config) -> Result<PkgStats, String>
	- 作用：对 pkg_temp 中的 .pkg 逐个解包到 unpacked_output_path，并复制关联资源。
- run_tex(config: &Config) -> Result<TexStats, String>
	- 作用：遍历解包目录中 .tex，输出 PNG/视频到 tex_converted（或自定义路径）。
- run_auto(config: &Config) -> Result<AutoStats, String>
	- 作用：按顺序执行 wallpaper → pkg → tex，并按配置选择清理临时目录。
- cleanup_temp(config: &Config)
	- 作用：删除 pkg_temp_path（无错误返回）。
- cleanup_unpacked(config: &Config)
	- 作用：当前为空操作/占位，避免误删。

返回数据结构（api::native 与 api::types）：
- WallpaperStats { raw_count, pkg_count }
- PkgStats { processed_files, extracted_files }
- TexStats { processed_files, converted_files }
- AutoStats { wallpaper: WallpaperStats, pkg: PkgStats, tex: TexStats }
- StatusCode = Success | Warning | Error
- OperationResult<T> { status, message, data: Option<T> }

## API 库（C FFI / 动态库）

入口位于 api::ffi，全部接受 UTF-8 JSON 配置字符串（Config 结构），返回 `char*` 指向的 JSON（需由调用方调用 lianpkg_free_string 释放）：
- lianpkg_run_wallpaper(config_json: *const c_char) -> *mut c_char
- lianpkg_run_pkg(config_json: *const c_char) -> *mut c_char
- lianpkg_run_tex(config_json: *const c_char) -> *mut c_char
- lianpkg_run_auto(config_json: *const c_char) -> *mut c_char
- lianpkg_free_string(s: *mut c_char)

返回 JSON 格式：OperationResult<T>，其中 T 为对应 Stats 结构；序列化失败时返回 `{"status":"Error","message":"JSON serialization failed","data":null}`。

## CLI 库（命令行接口）

命令定义（clap，见 cli::Cli）：
- 全局参数：
	- --config <FILE>: 指定配置文件路径（TOML），未提供则按默认位置加载/生成。
	- -d, --debug: 打开调试输出（带时间戳）。
- 子命令：
	- wallpaper [--search PATH] [--raw-out PATH] [--pkg-temp PATH] [--no-raw]
		- 覆盖对应 Config.wallpaper 字段；no-raw 关闭 raw 拷贝。
	- pkg [--input PATH] [--output PATH]
		- 覆盖 pkg_temp_path 与 unpacked_output_path。
	- tex [--input PATH] [--output PATH]
		- 覆盖 unpacked_output_path 与 tex.converted_output_path。
	- auto

执行流程（cli::run & cli::handlers）：
- 解析 CLI → 加载/合并配置 → 根据子命令调用 handlers::*。
- handlers::run_wallpaper / run_pkg / run_tex：打印标题，调用 api::native 对应函数，输出结果或错误。
- handlers::run_auto：
	- 预估磁盘占用（paper::estimate_requirements + human_bytes），可提示空间不足并等待确认。
	- 调用 api::native::run_auto；成功打印汇总，失败时执行 cleanup_temp 并打印错误报告。
- cli::logger：提供 info/success/error/title/debug 输出，可通过 set_debug 控制格式。

## 核心库（core）结构概览

- core::config: Config 结构与加载/合并逻辑，支持用户配置覆盖默认配置，缺失时写入示例配置到用户 config 目录。
- core::path: 路径工具（默认路径、展开 ~、Steam 路径探测、遍历收集 .pkg/.tex、输出路径生成、项目根探测）。
- core::paper: 壁纸扫描与复制；统计 raw/pkg 数；可估算磁盘占用。
- core::pkg: 简单 TEXV/PKG 格式解包，按表写出文件。
- core::tex: TEX 读取与转换管线（reader 解析 → converter 解压/解码 → 保存 PNG/原始媒体）。

## 能力边界速览

- 输入：依赖 Config 中的路径设置；FFI 接口要求 UTF-8 JSON，调用方需负责字符串释放。
- 处理能力：
	- 壁纸：复制包含 .pkg 的文件夹到临时目录；非 pkg 壁纸可选择复制。
	- PKG：按内部索引无压缩解包；目录名重复会自动生成唯一输出路径。
	- TEX：支持 LZ4 解压；支持 DXT1/3/5、RGBA8888、RG88、R8，或直接保存已有图片/MP4 数据；未知格式返回错误。
- 清理：auto 模式可选清理 pkg_temp 与解包目录；cleanup_unpacked 目前为占位（防止误删）。
- 安全：路径不存在、JSON/UTF-8 解析、文件 IO、格式校验均返回字符串错误；FFI 返回结构化 JSON 错误信息。
