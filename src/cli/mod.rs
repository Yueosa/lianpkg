use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::config::{self, Config};
use crate::config::ConfigStatus;
use crate::{commands, log};

#[derive(Parser, Debug)]
#[command(name = "lianpkg", version, about = "Steam wallpaper extract & convert tool")]
pub struct Cli {
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(short, long)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Wallpaper {
        #[arg(value_name = "SEARCH", index = 1)]
        search_pos: Option<String>,
        #[arg(value_name = "RAW_OUT", index = 2)]
        raw_pos: Option<String>,
        #[arg(value_name = "PKG_TEMP", index = 3)]
        pkg_temp_pos: Option<String>,

        #[arg(long, value_name = "PATH")]
        search: Option<String>,
        #[arg(long = "raw-out", value_name = "PATH")]
        raw_output: Option<String>,
        #[arg(long = "pkg-temp", value_name = "PATH")]
        pkg_temp: Option<String>,

        #[arg(long = "no-raw")]
        no_raw: bool,
    },

    Pkg {
        #[arg(value_name = "INPUT", index = 1)]
        input_pos: Option<String>,
        #[arg(value_name = "OUTPUT", index = 2)]
        output_pos: Option<String>,

        #[arg(long, value_name = "PATH")]
        input: Option<String>,
        #[arg(long, value_name = "PATH")]
        output: Option<String>,
    },

    Tex {
        #[arg(value_name = "INPUT", index = 1)]
        input_pos: Option<String>,

        #[arg(long, value_name = "PATH")]
        input: Option<String>,
        #[arg(long = "output", value_name = "PATH")]
        output: Option<String>,
    },

    Auto {
        #[arg(long, value_name = "PATH")]
        search: Option<String>,
        #[arg(long = "raw-out", value_name = "PATH")]
        raw_output: Option<String>,
        #[arg(long = "pkg-temp", value_name = "PATH")]
        pkg_temp: Option<String>,
        #[arg(long = "input", value_name = "PATH")]
        input: Option<String>,
        #[arg(long = "unpacked-out", value_name = "PATH")]
        unpacked_output: Option<String>,
        #[arg(long = "tex-out", value_name = "PATH")]
        tex_output: Option<String>,

        #[arg(long = "no-raw")]
        no_raw: bool,

        #[arg(long = "no-clean-temp")]
        no_clean_temp: bool,
        #[arg(long = "no-clean-unpacked")]
        no_clean_unpacked: bool,

        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}

#[derive(Default)]
struct Overrides {
    workshop_path: Option<String>,
    raw_output_path: Option<String>,
    pkg_temp_path: Option<String>,
    unpacked_output_path: Option<String>,
    tex_output_path: Option<String>,
    clean_pkg_temp: Option<bool>,
    clean_unpacked: Option<bool>,
    enable_raw_output: Option<bool>,
}

pub fn run_cli() {
    let cli = Cli::parse();
    run(cli);
}

fn run(cli: Cli) {
    log::set_debug(cli.debug);

    let mut config = match config::load_config(cli.config.clone()) {
        ConfigStatus::Loaded(c) => c,
        ConfigStatus::CreatedDefault(path) => {
            println!("首次运行，已生成默认配置: {:?}", path);
            println!("如需自定义请编辑配置后重新运行。");
            pause_windows();
            return;
        }
        ConfigStatus::Error(e) => {
            eprintln!("[ERROR] {}", e);
            return;
        }
    };

    let command = cli.command.unwrap_or(Command::Auto {
        search: None,
        raw_output: None,
        pkg_temp: None,
        input: None,
        unpacked_output: None,
        tex_output: None,
        no_clean_temp: false,
        no_clean_unpacked: false,
        no_raw: false,
        dry_run: false,
    });

    let overrides = collect_overrides(&command);
    apply_overrides(&mut config, overrides);

    if let Command::Auto { dry_run: true, .. } = command {
        print_dry_run(&config);
        pause_windows();
        return;
    }

    match command {
        Command::Wallpaper { .. } => {
            if let Err(e) = commands::run_wallpaper(&config) {
                log::error(&format!("Wallpaper extraction failed: {}", e));
            }
        }
        Command::Pkg { .. } => {
            if let Err(e) = commands::run_pkg(&config) {
                log::error(&format!("PKG unpacking failed: {}", e));
            }
        }
        Command::Tex { .. } => {
            if let Err(e) = commands::run_tex(&config) {
                log::error(&format!("TEX conversion failed: {}", e));
            }
        }
        Command::Auto { .. } => {
            commands::run_auto(&config);
        }
    }

    pause_windows();
}

fn collect_overrides(command: &Command) -> Overrides {
    let mut ov = Overrides::default();

    match command {
        Command::Wallpaper { search, search_pos, raw_output, raw_pos, pkg_temp, pkg_temp_pos, no_raw } => {
            ov.workshop_path = search.clone().or_else(|| search_pos.clone());
            ov.raw_output_path = raw_output.clone().or_else(|| raw_pos.clone());
            ov.pkg_temp_path = pkg_temp.clone().or_else(|| pkg_temp_pos.clone());
            if *no_raw { ov.enable_raw_output = Some(false); }
        }
        Command::Pkg { input, input_pos, output, output_pos } => {
            ov.pkg_temp_path = input.clone().or_else(|| input_pos.clone());
            ov.unpacked_output_path = output.clone().or_else(|| output_pos.clone());
        }
        Command::Tex { input, input_pos, output } => {
            ov.unpacked_output_path = input.clone().or_else(|| input_pos.clone());
            ov.tex_output_path = output.clone();
        }
        Command::Auto { search, raw_output, pkg_temp, input, unpacked_output, tex_output, no_clean_temp, no_clean_unpacked, no_raw, .. } => {
            ov.workshop_path = search.clone();
            ov.raw_output_path = raw_output.clone();
            ov.pkg_temp_path = pkg_temp.clone();
            ov.unpacked_output_path = unpacked_output.clone().or_else(|| input.clone());
            ov.tex_output_path = tex_output.clone();
            if *no_clean_temp { ov.clean_pkg_temp = Some(false); }
            if *no_clean_unpacked { ov.clean_unpacked = Some(false); }
            if *no_raw { ov.enable_raw_output = Some(false); }
        }
    }

    ov
}

fn apply_overrides(config: &mut Config, ov: Overrides) {
    if let Some(v) = ov.workshop_path { config.wallpaper.workshop_path = v; }
    if let Some(v) = ov.raw_output_path { config.wallpaper.raw_output_path = v; }
    if let Some(v) = ov.pkg_temp_path { config.wallpaper.pkg_temp_path = v; }
    if let Some(v) = ov.unpacked_output_path { config.unpack.unpacked_output_path = v; }
    if let Some(v) = ov.tex_output_path { config.tex.converted_output_path = Some(v); }
    if let Some(v) = ov.clean_pkg_temp { config.unpack.clean_pkg_temp = v; }
    if let Some(v) = ov.clean_unpacked { config.unpack.clean_unpacked = v; }
    if let Some(v) = ov.enable_raw_output { config.wallpaper.enable_raw_output = v; }
}

fn print_dry_run(config: &Config) {
    println!("=== Dry Run / 仅预览 ===");
    println!("wallpaper.search_path      => {:?}", config.wallpaper.workshop_path);
    println!("wallpaper.raw_output_path  => {:?}", config.wallpaper.raw_output_path);
    println!("wallpaper.pkg_temp_path    => {:?}", config.wallpaper.pkg_temp_path);
    println!("wallpaper.enable_raw_output=> {}", config.wallpaper.enable_raw_output);
    println!("unpack.unpacked_output     => {:?}", config.unpack.unpacked_output_path);
    println!("tex.converted_output_path  => {:?}", config.tex.converted_output_path.as_deref().unwrap_or("<默认 tex_converted>"));
    println!("clean_pkg_temp             => {}", config.unpack.clean_pkg_temp);
    println!("clean_unpacked             => {}", config.unpack.clean_unpacked);
}

fn pause_windows() {
    #[cfg(target_os = "windows")]
    {
        println!("\n按 Enter 退出...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
