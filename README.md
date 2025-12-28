# LianPkg ✨

LianPkg 是一个用于处理 Wallpaper Engine 壁纸资源的综合工具。它可以提取壁纸文件、解包 `.pkg` 文件以及将 `.tex` 纹理转换为常见的图像格式，支持 Linux 与 Windows。

---

## 使用前须知 ⚠️

> 推荐先执行一次 `lianpkg auto --dry-run` 核对实际路径

LianPkg 的工作对象是 Steam Workshop 中的 Wallpaper Engine 壁纸资源，默认处理目录为：

- **Linux**: `~/.local/share/Steam/steamapps/workshop/content/431960`
- **Windows**: 自动扫描 `libraryfolders.vdf` 定位

程序会自动扫描 Steam 库配置文件，即使你的 Wallpaper Engine 安装在非默认的 Steam 库，程序也能自动定位到正确的壁纸路径。

**前提条件**：
- 已安装并运行过 Steam 官方的 Wallpaper Engine
- 已通过 Steam 订阅并下载过壁纸

---

## 安装 📦

### 下载预编译版本

在 [Releases](https://github.com/YourRepo/lianpkg/releases) 页面下载对应平台的二进制文件。

### Arch Linux (AUR)

```bash
yay -S lianpkg-bin
# 或
paru -S lianpkg-bin
```

### 从源码编译

```bash
git clone https://github.com/YourRepo/lianpkg.git
cd lianpkg
cargo build --release
# 二进制文件位于 target/release/lianpkg
```

---

## 配置 🛠️

首次运行时，LianPkg 会生成默认配置文件：

| 平台    | 配置路径                        |
| ------- | ------------------------------- |
| Linux   | `~/.config/lianpkg/config.toml` |
| Windows | `exe路径\config\config.toml`    |

配置优先级：**命令行参数** > `config.toml` > **默认值**

---

## 快速开始 🚀

```bash
# 一键处理所有壁纸（推荐）
lianpkg auto

# 先预览将执行的操作
lianpkg auto --dry-run

# 增量处理（跳过之前已处理的壁纸）
lianpkg auto --incremental
```

---

## 命令参考 📖

> 此部分面向高级用户

```
lianpkg [OPTIONS] <COMMAND>
```

### 全局选项

| 选项                  | 说明             |
| --------------------- | ---------------- |
| `-c, --config <FILE>` | 指定配置文件路径 |
| `-d, --debug`         | 启用调试日志     |
| `-h, --help`          | 显示帮助信息     |
| `-V, --version`       | 显示版本信息     |

### 命令列表

| 命令        | 别名 | 说明           |
| ----------- | ---- | -------------- |
| `wallpaper` | `w`  | 壁纸扫描与复制 |
| `pkg`       | `p`  | PKG 文件解包   |
| `tex`       | `t`  | TEX 文件转换   |
| `auto`      | `a`  | 全自动流水线   |
| `config`    | `c`  | 配置管理       |
| `status`    | `s`  | 状态查看       |

---

### `wallpaper` — 壁纸扫描与复制 🖼️

扫描 Steam Workshop 目录，将壁纸分类提取。

```bash
lianpkg wallpaper [OPTIONS] [PATH]
```

**参数**：
- `[PATH]` — 壁纸源目录（默认从配置读取）

**选项**：
| 短格式 | 长格式              | 说明                             |
| ------ | ------------------- | -------------------------------- |
| `-r`   | `--raw-out <PATH>`  | 原始壁纸输出路径                 |
| `-t`   | `--pkg-temp <PATH>` | PKG 临时输出路径                 |
|        | `--no-raw`          | 跳过原始壁纸复制（只提取 PKG）   |
| `-i`   | `--ids <IDS>`       | 只处理指定壁纸 ID（逗号分隔）    |
| `-p`   | `--preview`         | 预览模式（列出壁纸，不执行复制） |
| `-v`   | `--verbose`         | 详细预览（显示完整元数据）       |

**示例**：
```bash
# 预览所有壁纸
lianpkg wallpaper --preview

# 只提取特定壁纸
lianpkg wallpaper --ids 123456789,987654321
# 或使用短格式
lianpkg w -i 123456789,987654321

# 自定义输出路径
lianpkg wallpaper -r ~/wallpapers/raw -t ~/wallpapers/pkg
```

---

### `pkg` — PKG 文件解包 📦

将 `.pkg` 文件解包为原始资源（纹理、JSON 等）。

```bash
lianpkg pkg [OPTIONS] [PATH]
```

**参数**：
- `[PATH]` — 输入路径（.pkg 文件、壁纸目录或 Pkg_Temp 目录）

**选项**：
| 短格式 | 长格式            | 说明                              |
| ------ | ----------------- | --------------------------------- |
| `-o`   | `--output <PATH>` | 解包输出路径                      |
| `-p`   | `--preview`       | 预览模式（显示 PKG 内容，不解包） |
| `-v`   | `--verbose`       | 详细预览                          |

**示例**：
```bash
# 解包单个 PKG 文件
lianpkg pkg ./scene.pkg -o ./output

# 预览 PKG 内容
lianpkg pkg ./scene.pkg -p -V

# 批量解包目录
lianpkg p ~/wallpapers/pkg_temp
```

---

### `tex` — TEX 文件转换 🧩

将 `.tex` 纹理文件转换为 PNG/图像格式。

```bash
lianpkg tex [OPTIONS] [PATH]
```

**参数**：
- `[PATH]` — 输入路径（.tex 文件或包含 .tex 的目录）

**选项**：
| 短格式 | 长格式            | 说明                                                      |
| ------ | ----------------- | --------------------------------------------------------- |
| `-o`   | `--output <PATH>` | 转换输出路径（默认在源文件同级生成 `tex_converted` 目录） |
| `-p`   | `--preview`       | 预览模式（显示 TEX 格式信息，不转换）                     |
| `-v`   | `--verbose`       | 详细预览                                                  |

**示例**：
```bash
# 转换单个 TEX 文件
lianpkg tex ./texture.tex

# 预览 TEX 格式信息
lianpkg t ./texture.tex -p -V

# 批量转换目录
lianpkg tex ~/wallpapers/unpacked -o ~/wallpapers/images
```

---

### `auto` — 全自动流水线 🤖

按顺序执行：**提取壁纸** → **解包 PKG** → **转换 TEX**

```bash
lianpkg auto [OPTIONS]
```

**路径选项**：
| 短格式 | 长格式                  | 说明                   |
| ------ | ----------------------- | ---------------------- |
| `-q`   | `--quiet`               | 静默模式（只输出结果） |
| `-s`   | `--search <PATH>`       | 壁纸源目录             |
| `-r`   | `--raw-out <PATH>`      | 原始壁纸输出目录       |
| `-t`   | `--pkg-temp <PATH>`     | PKG 临时目录           |
| `-u`   | `--unpacked-out <PATH>` | 解包输出目录           |
| `-o`   | `--tex-out <PATH>`      | TEX 转换输出目录       |

**行为选项**：
| 短格式 | 长格式                | 说明                          |
| ------ | --------------------- | ----------------------------- |
|        | `--no-raw`            | 跳过原始壁纸提取              |
|        | `--no-tex`            | 跳过 TEX 转换                 |
|        | `--no-clean-temp`     | 保留 PKG 临时目录             |
|        | `--no-clean-unpacked` | 保留解包中间产物              |
| `-I`   | `--incremental`       | 增量处理（跳过已处理的壁纸）  |
| `-i`   | `--ids <IDS>`         | 只处理指定壁纸 ID（逗号分隔） |
| `-n`   | `--dry-run`           | 仅显示计划，不执行            |

**示例**：
```bash
# 一键处理所有壁纸
lianpkg auto
# 或使用短命令
lianpkg a

# 预览执行计划
lianpkg auto -n

# 增量处理新壁纸
lianpkg auto -I

# 只处理特定壁纸
lianpkg a -i 123456789

# 保留中间文件用于调试
lianpkg auto --no-clean-temp --no-clean-unpacked

# 自定义输出路径
lianpkg auto -s ~/workshop -o ~/output/converted
```

---

### `config` — 配置管理 ⚙️

管理 LianPkg 配置文件。

```bash
lianpkg config <SUBCOMMAND>
```

**子命令**：
| 命令                | 说明                    |
| ------------------- | ----------------------- |
| `show`              | 显示当前完整配置        |
| `path`              | 显示配置文件路径        |
| `get <KEY>`         | 获取指定配置项          |
| `set <KEY> <VALUE>` | 设置配置项              |
| `reset [-y]`        | 重置为默认配置          |
| `edit`              | 用 $EDITOR 打开配置文件 |

**示例**：
```bash
# 查看当前配置
lianpkg config show

# 修改配置项
lianpkg config set wallpaper.workshop_path "/custom/path"

# 编辑配置文件
lianpkg config edit
```

---

### `status` — 状态查看 📊

查看处理状态和统计信息。

```bash
lianpkg status [OPTIONS]
```

**选项**：
| 选项        | 说明                        |
| ----------- | --------------------------- |
| `--full`    | 显示完整统计                |
| `--list`    | 列出所有已处理的壁纸        |
| `--clear`   | 清除状态记录                |
| `-y, --yes` | 跳过确认（与 --clear 配合） |

**示例**：
```bash
# 查看处理状态
lianpkg status

# 列出已处理壁纸
lianpkg status --list

# 清除状态（重新处理）
lianpkg status --clear -y
```

---

## 磁盘空间预估 💾

执行 `auto` 模式时，程序会自动：

1. **预估磁盘占用** — 根据 PKG 文件大小估算峰值空间需求
2. **检查剩余空间** — 空间不足时警告并等待确认
3. **错误保护** — 发生错误时自动清理临时文件

---

## 免责声明 📄

本工具仅供学习交流和个人备份使用。

1. **版权归属**: 本工具提取的所有资源版权归原作者或 Wallpaper Engine 所有。请勿用于商业用途。
2. **使用责任**: 用户应遵守相关法律法规，开发者不承担任何使用后果责任。
3. **非官方工具**: 本项目与 Wallpaper Engine 或 Valve (Steam) 无任何官方关联。

---

## 致谢 🙏

本项目算法灵感来源于对现有工具的研究：

- **[RePKG](https://github.com/notscuffed/repkg)** by notscuffed (MIT License) — PKG 文件结构参考
- **[we](https://github.com/redpfire/we)** by redpfire (GPL-3.0 License) — 文件格式分析参考

LianPkg 是完全独立的 Rust 重写版本，未复制任何源代码。

---

## License

MIT License
