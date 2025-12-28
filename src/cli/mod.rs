//! CLI 模块 - 命令行入口
//!
//! 解析命令行参数并分发到对应的处理器

pub mod args;
pub mod output;
pub mod logger;
pub mod handlers;

use clap::Parser;
use args::{Cli, Command};

/// CLI 入口函数
pub fn run() {
    let cli = Cli::parse();

    // 设置调试模式
    logger::set_debug(cli.debug);

    // 获取配置路径
    let config_path = cli.config.clone();
    // 保存一份用于最后显示
    let config_path_for_display = config_path.clone();

    // 分发命令
    let result = match cli.command {
        Some(Command::Wallpaper(ref args)) => {
            handlers::wallpaper::run(args, config_path)
        }
        Some(Command::Pkg(ref args)) => {
            handlers::pkg::run(args, config_path)
        }
        Some(Command::Tex(ref args)) => {
            handlers::tex::run(args, config_path)
        }
        Some(Command::Auto(ref args)) => {
            handlers::auto::run(args, config_path)
        }
        Some(Command::Config(ref args)) => {
            handlers::config::run(args, config_path)
        }
        Some(Command::Status(ref args)) => {
            handlers::status::run(args, config_path)
        }
        None => {
            // Windows 下无参数时，默认执行 auto 模式
            #[cfg(target_os = "windows")]
            {
                if !cli.quiet {
                    output::info("No command specified, running in auto mode...");
                    println!();
                }
                let auto_args = args::AutoArgs::default();
                handlers::auto::run(&auto_args, config_path)
            }
            #[cfg(not(target_os = "windows"))]
            {
                // Linux 下显示帮助
                use clap::CommandFactory;
                let _ = Cli::command().print_help();
                println!();
                Ok(())
            }
        }
    };

    // 处理错误
    if let Err(ref e) = result {
        output::error(e);
        std::process::exit(1);
    }

    // Windows 下等待用户确认（显示配置文件路径）
    output::press_enter_to_exit_with_config(config_path_for_display.as_deref());
}

