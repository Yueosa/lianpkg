# LianPkg ✨

LianPkg 是一个用于处理 Wallpaper Engine 壁纸资源的综合工具。它可以提取壁纸文件、解包 `.pkg` 文件以及将 `.tex` 纹理转换为常见的图像格式，支持 Linux 与 Windows。

---

## 使用前须知（重要）⚠️

由于 LianPkg 的工作对象是 Steam Workshop 中的 Wallpaper Engine 壁纸资源，默认处理目录为：

```swift
~/.local/share/Steam/steamapps/workshop/content/431960
```

程序会自动扫描 `libraryfolders.vdf`，即使你的 Wallpaper Engine 安装在非默认的 Steam 库（如 D 盘、E 盘），程序也能自动定位到正确的壁纸路径，无需手动配置！

因此，在使用本工具之前，请确保你的机器上 已经安装并至少运行过一次来自 Linux Steam 官方的 Wallpaper Engine，并且已经通过 Steam 下载过壁纸内容。

如果本地不存在上述目录或目录为空，LianPkg 将无法找到任何可处理的壁纸资源 🐾

> 其实你也可以自行下载 `.pkg` 文件，并修改程序的处理路径，但本程序推荐您在官方 Wallpaper Engine 获取安全的壁纸文件

## 安装 📦

你可以直接在 Releases 页面下载编译好的二进制文件使用（提供 Linux 与 Windows 版本）。

或者，如果你想自己编译，请确保你已经安装了 Rust 和 Cargo：

```bash
cargo build --release
```

编译后的二进制文件位于 `target/release/lianpkg`。

#### ArchLinux

可以通过AUR安装 [lianpkg-bin](https://aur.archlinux.org/packages/lianpkg-bin)

```shell
yay -S lianpkg-bin

#或使用paru
paru -S lianpkg-bin
```

---

## 配置 🛠️

首次运行时，LianPkg 会生成默认配置文件：
- Linux: `~/.config/lianpkg/default.toml`
- Windows: 与 exe 同目录（便携化；可用 `--config` 指定其他路径）

你可以在同一目录下创建 `config.toml` 来覆盖特定设置。程序的配置优先级如下：
1. **命令行参数**
2. `config.toml`
3. `default.toml`
4. **硬编码默认值**

---

## 使用说明 🚀

```bash
lianpkg <模式> [全局选项] [子命令选项]
```

### 全局选项 🌍

- `--config <FILE>`：指定配置文件路径（不写则使用默认路径，首次运行会自动生成配置）
- `-d, --debug`：启用调试日志
- `-h, --help`：查看帮助

### 模式与参数 🧭

所有子命令同时支持位置参数（兼容旧用法）和长参数（更易读）。

1) `wallpaper` — 提取壁纸 🖼️

```bash
lianpkg wallpaper [SEARCH] [RAW_OUT] [PKG_TEMP]
  --search <PATH> --raw-out <PATH> --pkg-temp <PATH> [--no-raw]
```

- `SEARCH`/`--search`：壁纸源目录（对应 `wallpaper.workshop_path`），默认指向 Steam Workshop 目录。
- `RAW_OUT`/`--raw-out`：无需解包的壁纸输出目录（`wallpaper.raw_output_path`）。
- `PKG_TEMP`/`--pkg-temp`：临时存放 `.pkg` 的目录（`wallpaper.pkg_temp_path`），解包结束可按清理开关删除。
- `--no-raw`：不提取非 PKG 的原始壁纸。

2) `pkg` — 解包 `.pkg` 📦

```bash
lianpkg pkg [INPUT] [OUTPUT]
  --input <PATH> --output <PATH>
```

- `INPUT`/`--input`：读取 `.pkg` 的目录，默认使用 `wallpaper.pkg_temp_path`。
- `OUTPUT`/`--output`：首次解包产物目录（`unpack.unpacked_output_path`）。

3) `tex` — 转换 `.tex` 🧩

```bash
lianpkg tex [INPUT]
  --input <PATH> --output <PATH>
```

`--output` 会覆盖 `tex.converted_output_path`，不写则使用默认的 `tex_converted` 结构。

- `INPUT`/`--input`：查找 `.tex` 的目录，默认使用 `unpack.unpacked_output_path`。
- `--output`：最终图片输出目录；留空则在每个场景下创建 `tex_converted`，保持原有层级。

4) `auto` — 一键执行 wallpaper -> pkg -> tex 🤖

```bash
lianpkg auto \
  [--search <PATH>] [--raw-out <PATH>] [--pkg-temp <PATH>] \
  [--input <PATH>] [--unpacked-out <PATH>] [--tex-out <PATH>] \
  [--no-clean-temp] [--no-clean-unpacked] [--no-raw] [--dry-run]
```

- `--no-clean-temp`：结束时保留 `Pkg_Temp`
- `--no-clean-unpacked`：保留 `Pkg_Unpacked`（仍保留 `tex_converted`）
- `--no-raw`：跳过提取非 PKG 的原始壁纸（节省空间）
- `--dry-run`：仅打印解析后的路径与清理计划，不执行任何操作

- `--search`/`--raw-out`/`--pkg-temp`：覆盖 wallpaper 阶段路径。
- `--input`/`--unpacked-out`：覆盖 pkg 解包输入/输出路径。
- `--tex-out`：覆盖 tex 最终输出目录；默认使用 `tex_converted` 结构。
- `--no-clean-temp` 对应 `unpack.clean_pkg_temp`；`--no-clean-unpacked` 对应 `unpack.clean_unpacked`。

### 磁盘空间预估与检查 💾

在执行 `auto` 模式时，程序会自动：
1. **预估磁盘占用**：根据扫描到的 PKG 文件大小，预估解包和转换所需的峰值空间。
2. **检查剩余空间**：如果目标磁盘空间不足，程序会发出警告并暂停，等待用户确认。
3. **错误保护**：如果执行过程中发生错误，程序会自动清理已生成的临时文件，防止残留垃圾。

### 平台差异与运行方式 🖥️

- Linux：默认路径基于 `~/.local/share`，命令行运行 `lianpkg ...`。
- Windows：默认路径基于 exe 同目录（便携化）；可双击运行，流程结束会提示按 Enter 退出；也可在 cmd/PowerShell 运行 `lianpkg.exe ...`。

### 重要提示 📌

- 默认路径可通过 CLI 覆盖或修改 `config.toml`。
- 清理开关默认开启（保留 `tex_converted`），可用 `--no-clean-temp` / `--no-clean-unpacked` 关闭。

### 快速上手示例 ⏱️

- 一键执行默认流程：`lianpkg auto`
- 仅解包自定义路径：`lianpkg pkg /path/to/pkg /tmp/output`
- 转换 `.tex` 并指定最终输出：`lianpkg tex /tmp/Pkg_Unpacked --output /tmp/tex_converted`
- 只想看看会做什么：`lianpkg auto --dry-run`

---

## 免责声明 (Disclaimer) 📄

本工具仅供学习交流和个人备份使用。

1. **版权归属**: 本工具提取和解包的所有资源（包括但不限于图片、视频、脚本等）的版权归原作者或 Wallpaper Engine 所有。请勿将提取的资源用于商业用途或违反原作者许可的用途。
2. **使用责任**: 用户在使用本工具时应遵守相关法律法规。对于用户使用本工具所产生的任何后果（包括但不限于版权纠纷、数据丢失等），开发者不承担任何责任。
3. **非官方工具**: 本项目与 Wallpaper Engine 或 Valve (Steam) 没有任何官方关联。

---

## 致谢与参考资料 🙏

本项目灵感来源于对现有工具的研究，这些工具用于处理 Wallpaper Engine 的资源格式。

- **RePKG**，作者 notscuffed（MIT 许可证）

用作理解 `.pkg` 文件结构的参考。

- **we**，作者 redpfire（GPL-3.0 许可证）

用于文件格式分析和解包逻辑。

本项目未复制任何源代码；LianPkg 是一个完全独立于源代码的 Rust 重写版本。

本项目与 Wallpaper Engine、Valve 或上述工具的作者均无任何关联。
