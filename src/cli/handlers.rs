use lianpkg::core::config::Config;
use crate::cli::logger as log;
use lianpkg::api::native;
use lianpkg::core::{path, paper};
use human_bytes::human_bytes;

pub fn run_wallpaper(config: &Config) -> Result<(), String> {
    log::title("Starting Wallpaper Extraction");
    match native::run_wallpaper(config) {
        Ok(stats) => {
            log::success(&format!("Wallpaper extraction completed. Raw: {}, Pkg: {}", stats.raw_count, stats.pkg_count));
            Ok(())
        },
        Err(e) => Err(e)
    }
}

pub fn run_pkg(config: &Config) -> Result<(), String> {
    log::title("🚀 Starting PKG Unpack");
    match native::run_pkg(config) {
        Ok(stats) => {
            log::success(&format!("PKG unpack completed. Processed: {}, Extracted: {}", stats.processed_files, stats.extracted_files));
            Ok(())
        },
        Err(e) => Err(e)
    }
}

pub fn run_tex(config: &Config) -> Result<(), String> {
    log::title("🚀 Starting TEX Conversion");
    match native::run_tex(config) {
        Ok(stats) => {
            log::success(&format!("TEX conversion completed. Processed: {}, Converted: {}", stats.processed_files, stats.converted_files));
            Ok(())
        },
        Err(e) => Err(e)
    }
}

pub fn run_auto(config: &Config) {
    log::title("🤖 Starting Auto Mode");

    let search_path = path::expand_path(&config.wallpaper.workshop_path);
    log::info("Calculating estimated disk usage...");
    let (pkg_size, raw_size) = paper::estimate_requirements(&search_path, config.wallpaper.enable_raw_output);
    
    let est_pkg_temp = pkg_size;
    let est_unpacked = (pkg_size as f64 * 1.5) as u64;
    let est_converted = (pkg_size as f64 * 2.0) as u64;
    let est_raw = raw_size;

    let peak_usage = est_pkg_temp + est_unpacked + est_converted + est_raw;
    let final_usage = est_raw + est_converted + 
        if config.unpack.clean_unpacked { 0 } else { est_unpacked } +
        if config.unpack.clean_pkg_temp { 0 } else { est_pkg_temp };

    println!("==========================================");
    println!("          Disk Usage Estimation           ");
    println!("==========================================");
    println!("Found PKG Files:      {}", human_bytes(pkg_size as f64));
    if config.wallpaper.enable_raw_output {
        println!("Found Raw Files:      {}", human_bytes(raw_size as f64));
    } else {
        println!("Raw Files:            Skipped (--no-raw)");
    }
    println!("------------------------------------------");
    println!("Estimated Peak Usage: {}", human_bytes(peak_usage as f64));
    println!("Estimated Final Usage:{}", human_bytes(final_usage as f64));
    
    let output_path = path::expand_path(&config.unpack.unpacked_output_path);
    let mut check_path = output_path.clone();
    while !check_path.exists() {
        if let Some(parent) = check_path.parent() {
            check_path = parent.to_path_buf();
        } else {
            break;
        }
    }
    
    if check_path.exists() {
        match fs2::available_space(&check_path) {
            Ok(available) => {
                println!("Available Space:      {}", human_bytes(available as f64));
                if available < peak_usage {
                    log::error("⚠️  WARNING: Insufficient disk space! ⚠️");
                    println!("You might run out of space during processing.");
                    println!("Required: {}, Available: {}", human_bytes(peak_usage as f64), human_bytes(available as f64));
                    println!("Press Ctrl+C to cancel or Enter to continue anyway...");
                    let _ = std::io::stdin().read_line(&mut String::new());
                } else {
                    println!("Disk Space Check:     OK ✅");
                }
            },
            Err(_) => {
                println!("Available Space:      Unknown (Check failed)");
            }
        }
    }
    println!("==========================================");
    
    match native::run_auto(config) {
        Ok(stats) => {
            log::title("✨ Auto Mode Completed ✨");
            println!("==========================================");
            println!("             Summary Report               ");
            println!("==========================================");
            println!("Wallpaper Extraction:");
            println!("  - Raw Wallpapers:   {}", stats.wallpaper.raw_count);
            println!("  - PKGs Extracted:   {}", stats.wallpaper.pkg_count);
            println!("PKG Unpacking:");
            println!("  - PKGs Unpacked:    {}", stats.pkg.extracted_files);
            println!("TEX Conversion:");
            println!("  - TEXs Converted:   {}", stats.tex.converted_files);
            println!("==========================================");
        },
        Err(e) => {
            log::error(&format!("❌ Auto Mode Failed: {}", e));
            log::info("Performing emergency cleanup...");
            
            native::cleanup_temp(config);
            
            println!("==========================================");
            println!("             ERROR REPORT                 ");
            println!("==========================================");
            println!("An error occurred during execution:");
            println!("{}", e);
            println!("==========================================");
        }
    }
}
