# core 模块概览

## path
- 模块：`path`
- 模块简述：处理路径展开（含 `~`/环境变量）、提供配置/输出目录默认路径。
- 主要接口：
  - `expand_path(input: &str) -> PathBuf`：展开 `~`/环境变量，得到绝对路径。
  - `get_default_config_path() -> PathBuf`：返回配置文件默认路径。
  - `get_default_tex_output_path() -> PathBuf`：返回 TEX 输出目录默认路径。

## config
- 模块：`config`
- 模块简述：管理 TOML 配置和 state.json（处理进度标记）。
- 配置接口：
  - `create_config_file(path: &Path) -> Result<(), String>`：写入默认模板，若已有文件则覆盖。
  - `load_config(path: &Path) -> Result<Config, String>`：读取并解析配置为结构体。
  - `update_config(path: &Path, new_cfg: &ConfigRaw) -> Result<Config, String>`：按增量合并写回，并返回合并后的配置。
  - `delete_config_file(path: &Path) -> Result<(), String>`：删除单个配置文件。
  - `delete_config_dir(path: &Path) -> Result<(), String>`：删除包含配置的目录。
- 状态接口：
  - `load_state(path: &Path) -> Result<State, String>`：读取 state.json 为 State。
  - `save_state(path: &Path, state: &State) -> Result<(), String>`：写入 state.json。
  - `delete_state(path: &Path) -> Result<(), String>`：删除 state.json。
  - `mark_processed(path: &Path, flag: bool) -> Result<State, String>`：设置 processed 标记并落盘。
  - `clear_state(path: &Path) -> Result<State, String>`：清空 processed 标记并落盘。

## paper
- 模块：`paper`
- 模块简述：遍历和处理壁纸项目，读取元信息并输出处理结果/估算。
- 主要接口：
  - `list_workshop_dirs(base: &Path) -> Result<Vec<PathBuf>, String>`：列出 base 下符合规则的壁纸项目目录。
  - `read_project_meta(dir: &Path) -> Result<ProjectMeta, String>`：读取单个项目的 metadata 信息。
  - `process_folder(dir: &Path, output: &Path) -> Result<WallpaperStats, String>`：处理单个目录并返回统计数据。
  - `extract_wallpapers(base: &Path, output: &Path) -> Result<WallpaperStats, String>`：遍历多个目录逐个处理并汇总统计。
  - `estimate_requirements(dirs: &[PathBuf]) -> Result<FolderProcess, String>`：估算给定目录集合的空间/工作量需求。

## pkg
- 模块：`pkg`
- 模块简述：解包单个 `.pkg` 文件。
- 主要接口：
  - `unpack_pkg(file_path: &Path, output_base: &Path) -> Result<usize, String>`：解析条目表并写出每个文件，返回成功数量。

## tex
- 模块：`tex`
- 模块简述：解析并转存单个 TEX 文件。
- 主要接口：
  - `process_tex(input_path: &Path, output_path: &Path) -> Result<(), String>`：读 TEX、解码或直存，输出图片/视频/PNG。
