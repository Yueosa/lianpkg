# LianPkg ✨

LianPkg 是一个用于处理 Wallpaper Engine 壁纸资源的综合工具。它可以提取壁纸文件、解包 `.pkg` 文件以及将 `.tex` 纹理转换为常见的图像格式，支持 Linux 与 Windows。

---

## 使用前须知（重要）⚠️

###### 推荐先执行一次 `lianpkg auto --dry-run` 核对实际路径

由于 LianPkg 的工作对象是 Steam Workshop 中的 Wallpaper Engine 壁纸资源，默认处理目录(硬编码)为：

```swift
~/.local/share/Steam/steamapps/workshop/content/431960
```

工作环境下, 程序会自动扫描 `libraryfolders.vdf`，即使你的 Wallpaper Engine 安装在非默认的 Steam 库（如 D 盘、E 盘），程序也能自动定位到正确的壁纸路径，无需手动配置！

因此，在使用本工具之前，请确保你的机器上 已经安装并至少运行过一次来自 Steam 官方的 Wallpaper Engine，并且已经通过 Steam 下载过壁纸内容。

如果本地不存在上述目录或目录为空，LianPkg 将无法找到任何可处理的壁纸资源 🐾

> 其实你也可以自行下载 `.pkg` 文件，并修改程序的处理路径，但本程序推荐您在官方 Wallpaper Engine 获取安全的壁纸文件

## 安装 📦

#### 大部分 `Windows` `Linux`

你可以直接在 Releases 页面下载编译好的二进制文件使用（提供 Linux 与 Windows 版本）。

或者，如果你想自己编译，请确保你已经安装了 Rust 和 Cargo：

```bash
cargo build --release
```

编译后的二进制文件位于 `target/release/lianpkg`。

#### `ArchLinux`

除了直接下载二进制文件, 下载源码编译之外

你还可以通过AUR安装 [lianpkg-bin](https://aur.archlinux.org/packages/lianpkg-bin)

```shell
yay -S lianpkg-bin

#或使用paru
paru -S lianpkg-bin
```

---

## 配置 🛠️

首次运行时，LianPkg 会生成默认配置文件：
- Linux: `~/.config/lianpkg/config.toml`
- Windows: 与 exe 同目录（便携化；可用 `--config` 指定其他路径）

程序的配置优先级如下：
1. **命令行参数**
2. `config.toml`
3. **硬编码默认值**

---

## 使用说明 🚀

```bash
lianpkg [OPTIONS] <COMMAND>
```

### 全局选项 🌍

以下选项适用于所有模式，且必须位于子命令之前：

- `-c, --config <FILE>`：指定配置文件路径（默认使用自动生成的配置）
- `-d, --debug`：启用调试日志输出
- `-h, --help`：打印帮助信息
- `-V, --version`：打印版本信息

### 模式与参数 🧭

#### 1. `wallpaper` — 提取壁纸 🖼️

扫描 Steam Workshop 目录，将壁纸分类提取。

```bash
lianpkg wallpaper [OPTIONS] [SEARCH] [RAW_OUT] [PKG_TEMP]
```

**参数：**
- `[SEARCH]` / `--search <PATH>`  
  壁纸源目录（Steam Workshop 路径）。
- `[RAW_OUT]` / `--raw-out <PATH>`  
  无需解包的壁纸（如视频、网页）输出目录。
- `[PKG_TEMP]` / `--pkg-temp <PATH>`  
  `.pkg` 文件临时存放目录。

**选项：**
- `--no-raw`：仅提取 `.pkg` 文件，跳过普通壁纸的复制。

#### 2. `pkg` — 解包 `.pkg` 📦

将 `.pkg` 文件解包为原始资源（纹理、JSON 等）。

```bash
lianpkg pkg [OPTIONS] [INPUT] [OUTPUT]
```

**参数：**
- `[INPUT]` / `--input <PATH>`  
  输入路径，可以是单个 `.pkg` 文件或包含 `.pkg` 的目录。
- `[OUTPUT]` / `--output <PATH>`  
  解包产物的输出目录。

#### 3. `tex` — 转换 `.tex` 🧩

将 `.tex` 纹理文件转换为 PNG/图像格式。

```bash
lianpkg tex [OPTIONS] [INPUT]
```

**参数：**
- `[INPUT]` / `--input <PATH>`  
  输入路径，可以是单个 `.tex` 文件或包含 `.tex` 的目录。

**选项：**
- `--output <PATH>`  
  转换后的图片输出目录。如果不指定，默认在源文件同级生成 `tex_converted` 目录。

#### 4. `auto` — 一键全自动模式 🤖

按顺序执行：提取壁纸 -> 解包 PKG -> 转换纹理。

```bash
lianpkg auto [OPTIONS]
```

**路径覆盖选项：**
- `--search <PATH>`：壁纸源目录
- `--raw-out <PATH>`：原始壁纸输出目录
- `--pkg-temp <PATH>`： PKG 临时目录
- `--unpacked-out <PATH>` (或 `--input`)：解包中间产物目录
- `--tex-out <PATH>`：最终图片输出目录

**行为控制选项：**
- `--no-raw`：跳过原始壁纸提取
- `--no-clean-temp`：流程结束后保留 `Pkg_Temp` 目录
- `--no-clean-unpacked`：流程结束后保留 `Pkg_Unpacked` 目录（中间产物）
- `--dry-run`：仅打印将要使用的路径和配置，不执行实际操作


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

---

## 免责声明 (Disclaimer) 📄

本工具仅供学习交流和个人备份使用。

1. **版权归属**: 本工具提取和解包的所有资源（包括但不限于图片、视频、脚本等）的版权归原作者或 Wallpaper Engine 所有。请勿将提取的资源用于商业用途或违反原作者许可的用途。
2. **使用责任**: 用户在使用本工具时应遵守相关法律法规。对于用户使用本工具所产生的任何后果（包括但不限于版权纠纷、数据丢失等），开发者不承担任何责任。
3. **非官方工具**: 本项目与 Wallpaper Engine 或 Valve (Steam) 没有任何官方关联。

---

## 致谢与参考资料 🙏

本项目算法灵感来源于对现有工具的研究，这些工具用于处理 Wallpaper Engine 的资源格式。

- **RePKG**，作者 notscuffed（MIT 许可证）

用作理解 `.pkg` 文件结构的参考。

- **we**，作者 redpfire（GPL-3.0 许可证）

用于文件格式分析和解包逻辑。

本项目未复制任何源代码；LianPkg 是一个完全独立于源代码的 Rust 重写版本。

本项目与 Wallpaper Engine、Valve 或上述工具的作者均无任何关联。
