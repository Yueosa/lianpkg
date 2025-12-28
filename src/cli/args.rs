//! CLI 参数定义
//!
//! 使用 clap 定义所有命令行参数结构

use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;

/// LianPkg - Steam Wallpaper Engine 壁纸资源提取与转换工具
#[derive(Parser, Debug)]
#[command(
    name = "lianpkg",
    version,
    about = "Steam Wallpaper Engine 壁纸资源提取与转换工具",
    long_about = "LianPkg 是一个用于处理 Wallpaper Engine 壁纸资源的综合工具。\n\
                    它可以提取壁纸文件、解包 .pkg 文件以及将 .tex 纹理转换为常见的图像格式。"
)]
pub struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    /// 调试模式（显示详细日志）
    #[arg(short, long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// 子命令
#[derive(Subcommand, Debug)]
pub enum Command {
    /// 壁纸扫描与复制
    #[command(visible_alias = "w")]
    Wallpaper(WallpaperArgs),

    /// PKG 文件解包
    #[command(visible_alias = "p")]
    Pkg(PkgArgs),

    /// TEX 文件转换
    #[command(visible_alias = "t")]
    Tex(TexArgs),

    /// 全自动模式（流水线执行）
    #[command(visible_alias = "a")]
    Auto(AutoArgs),

    /// 配置管理
    #[command(visible_alias = "c")]
    Config(ConfigArgs),

    /// 状态查看
    #[command(visible_alias = "s")]
    Status(StatusArgs),
}

// ============================================================================
// Wallpaper 模式参数
// ============================================================================

#[derive(Args, Debug)]
pub struct WallpaperArgs {
    /// 壁纸源目录（默认从配置读取）
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// 原始壁纸输出路径
    #[arg(short = 'r', long = "raw-out", value_name = "PATH")]
    pub raw_output: Option<PathBuf>,

    /// PKG 临时输出路径
    #[arg(short = 't', long = "pkg-temp", value_name = "PATH")]
    pub pkg_temp: Option<PathBuf>,

    /// 跳过原始壁纸复制（只提取 PKG）
    #[arg(long = "no-raw")]
    pub no_raw: bool,

    /// 只处理指定的壁纸 ID（逗号分隔）
    #[arg(short = 'i', long, value_name = "IDS", value_delimiter = ',')]
    pub ids: Option<Vec<String>>,

    /// 预览模式（列出壁纸，不执行复制）
    #[arg(short = 'p', long)]
    pub preview: bool,

    /// 详细预览（显示完整元数据）
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

// ============================================================================
// PKG 模式参数
// ============================================================================

#[derive(Args, Debug)]
pub struct PkgArgs {
    /// 输入路径（.pkg 文件、壁纸目录或 Pkg_Temp 目录）
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// 解包输出路径
    #[arg(short = 'o', long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// 预览模式（显示 PKG 内容，不解包）
    #[arg(short = 'p', long)]
    pub preview: bool,

    /// 详细预览
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

// ============================================================================
// TEX 模式参数
// ============================================================================

#[derive(Args, Debug)]
pub struct TexArgs {
    /// 输入路径（.tex 文件或包含 .tex 的目录）
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// 转换输出路径
    #[arg(short = 'o', long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// 预览模式（显示 TEX 格式信息，不转换）
    #[arg(short = 'p', long)]
    pub preview: bool,

    /// 详细预览
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

// ============================================================================
/// Auto 模式参数
#[derive(Args, Debug, Default)]
pub struct AutoArgs {
    /// 壁纸源目录
    #[arg(short = 's', long, value_name = "PATH")]
    pub search: Option<PathBuf>,

    /// 原始壁纸输出路径
    #[arg(short = 'r', long = "raw-out", value_name = "PATH")]
    pub raw_output: Option<PathBuf>,

    /// PKG 临时目录
    #[arg(short = 't', long = "pkg-temp", value_name = "PATH")]
    pub pkg_temp: Option<PathBuf>,

    /// 解包输出目录
    #[arg(short = 'u', long = "unpacked-out", value_name = "PATH")]
    pub unpacked_output: Option<PathBuf>,

    /// TEX 转换输出目录
    #[arg(short = 'o', long = "tex-out", value_name = "PATH")]
    pub tex_output: Option<PathBuf>,

    /// 跳过原始壁纸提取
    #[arg(long = "no-raw")]
    pub no_raw: bool,

    /// 跳过 TEX 转换
    #[arg(long = "no-tex")]
    pub no_tex: bool,

    /// 保留 PKG 临时目录
    #[arg(long = "no-clean-temp")]
    pub no_clean_temp: bool,

    /// 保留解包中间产物
    #[arg(long = "no-clean-unpacked")]
    pub no_clean_unpacked: bool,

    /// 增量处理（跳过已处理的壁纸）
    #[arg(short = 'I', long)]
    pub incremental: bool,

    /// 只处理指定壁纸 ID（逗号分隔）
    #[arg(short = 'i', long, value_name = "IDS", value_delimiter = ',')]
    pub ids: Option<Vec<String>>,

    /// 仅显示计划执行的操作（不实际执行）
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// 精简输出模式（只显示关键信息）
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

// ============================================================================
// Config 模式参数
// ============================================================================

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: Option<ConfigCommand>,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// 显示当前完整配置
    Show,
    
    /// 显示配置文件路径
    Path,
    
    /// 获取指定配置项
    Get {
        /// 配置项键名（如 wallpaper.workshop_path）
        key: String,
    },
    
    /// 设置配置项
    Set {
        /// 配置项键名
        key: String,
        /// 配置项值
        value: String,
    },
    
    /// 重置为默认配置
    Reset {
        /// 跳过确认
        #[arg(long, short = 'y')]
        yes: bool,
    },
    
    /// 用 $EDITOR 打开配置文件
    Edit,
}

// ============================================================================
// Status 模式参数
// ============================================================================

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// 显示完整统计
    #[arg(long)]
    pub full: bool,

    /// 列出所有已处理的壁纸
    #[arg(long)]
    pub list: bool,

    /// 清除状态记录
    #[arg(long)]
    pub clear: bool,

    /// 跳过确认（与 --clear 配合）
    #[arg(long, short = 'y')]
    pub yes: bool,
}
