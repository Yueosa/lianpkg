use clap::{Parser, Subcommand};
use std::path::PathBuf;
use lianpkg::core::config::{self, Config, ConfigStatus};

pub mod handlers;
pub mod logger;

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
        #[arg(long, value_name = "PATH")]
        input: Option<String>,
        #[arg(long, value_name = "PATH")]
        output: Option<String>,
    },
    Tex {
        #[arg(long, value_name = "PATH")]
        input: Option<String>,
        #[arg(long, value_name = "PATH")]
        output: Option<String>,
    },
    Auto,
}

pub fn run() {
    let cli = Cli::parse();

    logger::set_debug(cli.debug);

    let mut config = match config::load_config(cli.config) {
        ConfigStatus::Loaded(c) => c,
        ConfigStatus::CreatedDefault(p) => {
            logger::info(&format!("Created default config at {:?}", p));
            Config::default()
        },
        ConfigStatus::Error(e) => {
            logger::error(&format!("Config error: {}", e));
            return;
        }
    };

    // Override config with CLI args
    if let Some(cmd) = &cli.command {
        match cmd {
            Command::Wallpaper { search, raw_output, pkg_temp, no_raw } => {
                if let Some(s) = search { config.wallpaper.workshop_path = s.clone(); }
                if let Some(r) = raw_output { config.wallpaper.raw_output_path = r.clone(); }
                if let Some(p) = pkg_temp { config.wallpaper.pkg_temp_path = p.clone(); }
                if *no_raw { config.wallpaper.enable_raw_output = false; }
                
                if let Err(e) = handlers::run_wallpaper(&config) {
                    logger::error(&format!("Wallpaper extraction failed: {}", e));
                }
            },
            Command::Pkg { input, output } => {
                if let Some(i) = input { config.wallpaper.pkg_temp_path = i.clone(); }
                if let Some(o) = output { config.unpack.unpacked_output_path = o.clone(); }

                if let Err(e) = handlers::run_pkg(&config) {
                    logger::error(&format!("PKG unpack failed: {}", e));
                }
            },
            Command::Tex { input, output } => {
                if let Some(i) = input { config.unpack.unpacked_output_path = i.clone(); }
                if let Some(o) = output { config.tex.converted_output_path = Some(o.clone()); }

                if let Err(e) = handlers::run_tex(&config) {
                    logger::error(&format!("TEX conversion failed: {}", e));
                }
            },
            Command::Auto => {
                handlers::run_auto(&config);
            }
        }
    } else {
        use clap::CommandFactory;
        let _ = Cli::command().print_help();
    }
}
