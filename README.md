# LianPkg

LianPkg 是一个用于处理 Wallpaper Engine 壁纸资源的综合工具。它可以提取壁纸文件、解包 `.pkg` 文件以及将 `.tex` 纹理转换为常见的图像格式。

---

## 使用前须知（重要）

由于 LianPkg 的工作对象是 Steam Workshop 中的 Wallpaper Engine 壁纸资源，它默认处理的目录为：

```swift
~/.local/share/Steam/steamapps/workshop/content/431960
```

因此，在使用本工具之前，请确保你的机器上 已经安装并至少运行过一次来自 Linux Steam 官方的 Wallpaper Engine，并且已经通过 Steam 下载过壁纸内容。

如果本地不存在上述目录或目录为空，LianPkg 将无法找到任何可处理的壁纸资源 🐾

> 其实你也可以自行下载 `.pkg` 文件，并修改程序的处理路径，但本程序推荐您在官方 Wallpaper Engine 获取安全的壁纸文件

## 安装

你可以直接在 Releases 页面下载编译好的二进制文件使用。

或者，如果你想自己编译，请确保你已经安装了 Rust 和 Cargo：

```bash
cargo build --release
```

编译后的二进制文件位于 `target/release/lianpkg`。

---

## 配置

首次运行时，LianPkg 会在以下位置生成默认配置文件：
`~/.config/lianpkg/default.toml`

你可以在同一目录下创建 `config.toml` 来覆盖特定设置。程序的配置优先级如下：
1. **命令行参数**
2. `config.toml`
3. `default.toml`
4. **硬编码默认值**

---

## 使用说明

```bash
lianpkg [模式] [选项]
```

### 1. `wallpaper` 模式

> 用于从 Wallpaper Engine 目录提取壁纸

```bash
lianpkg wallpaper [搜索路径] [输出路径]
```

- **参数 1 (可选): 搜索路径 (Search Path)**
    - 如果提供，将覆盖配置文件中的 `wallpaper.search_path`
    - 如果不提供，使用配置文件中的值
- **参数 2 (可选): 输出路径 (Output Path)**
    - 如果提供，将覆盖配置文件中的 `wallpaper.output_path`
    - 如果不提供，使用配置文件中的值

### 2. `pkg` 模式

> 用于解包 `.pkg` 文件

```bash
lianpkg pkg [输入路径] [输出路径]
```

- **参数 1 (可选): 输入路径 (Input Path)**
    - 如果提供，将作为查找 `.pkg` 文件的目录
    - 如果不提供，默认使用 `wallpaper` 模式输出路径下的 `Pkg` 文件夹 (即 `config.wallpaper.output_path/Pkg`)
- **参数 2 (可选): 输出路径 (Output Path)**
    - 如果提供，将覆盖配置文件中的 `pkg.output_path`
    - 如果不提供，使用配置文件中的值

### 3. `tex` 模式

> 用于将 `.tex` 文件转换为图片

```bash
lianpkg tex [输入路径]
```

- **参数 1 (可选): 输入路径 (Input Path)**
    - 如果提供，将作为查找 `.tex` 文件的目录
    - 如果不提供，默认使用 `pkg` 模式的输出路径 (即 `config.pkg.output_path`)
- **输出路径**: 此模式不支持通过命令行指定输出路径。它会自动在 `.tex` 文件所在的项目根目录下创建 `tex_converted` 文件夹

### 4. `auto` 模式

> 一键执行 `wallpaper` -> `pkg` -> `tex` 流程

```bash
lianpkg auto
```

- **参数**: 目前 `auto` 模式不接受任何路径参数
- 它完全依赖配置文件 (`config.toml` 或 `default.toml`) 中的路径设置来串联整个流程

---

## 常用选项

- `-h, --help`: 显示帮助信息
- `-d, --debug`: 启用调试日志输出

---

## 免责声明 (Disclaimer)

本工具仅供学习交流和个人备份使用。

1.  **版权归属**: 本工具提取和解包的所有资源（包括但不限于图片、视频、脚本等）的版权归原作者或 Wallpaper Engine 所有。请勿将提取的资源用于商业用途或违反原作者许可的用途。
2.  **使用责任**: 用户在使用本工具时应遵守相关法律法规。对于用户使用本工具所产生的任何后果（包括但不限于版权纠纷、数据丢失等），开发者不承担任何责任。
3.  **非官方工具**: 本项目与 Wallpaper Engine 或 Valve (Steam) 没有任何官方关联。

---

## 致谢与参考资料

本项目灵感来源于对现有工具的研究，这些工具用于处理 Wallpaper Engine 的资源格式。

- **RePKG**，作者 notscuffed（MIT 许可证）

用作理解 `.pkg` 文件结构的参考。

- **we**，作者 redpfire（GPL-3.0 许可证）

用于文件格式分析和解包逻辑。

本项目未复制任何源代码；LianPkg 是一个完全独立于源代码的 Rust 重写版本。

本项目与 Wallpaper Engine、Valve 或上述工具的作者均无任何关联。
