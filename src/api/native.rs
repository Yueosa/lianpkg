use crate::core::{config::Config, paper, pkg, tex, path};
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};

pub use crate::core::paper::WallpaperStats;


#[derive(Debug, Serialize, Deserialize)]
pub struct PkgStats {
    pub processed_files: usize,
    pub extracted_files: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TexStats {
    pub processed_files: usize,
    pub converted_files: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoStats {
    pub wallpaper: WallpaperStats,
    pub pkg: PkgStats,
    pub tex: TexStats,
}


pub fn run_wallpaper(config: &Config) -> Result<WallpaperStats, String> {
    let search_path = path::expand_path(&config.wallpaper.workshop_path);
    let raw_output_path = path::expand_path(&config.wallpaper.raw_output_path);
    let pkg_temp_path = path::expand_path(&config.wallpaper.pkg_temp_path);

    paper::extract_wallpapers(&search_path, &raw_output_path, &pkg_temp_path, config.wallpaper.enable_raw_output)
}


pub fn run_pkg(config: &Config) -> Result<PkgStats, String> {
    let input_path = path::expand_path(&config.wallpaper.pkg_temp_path);
    let output_path = path::expand_path(&config.unpack.unpacked_output_path);

    if !input_path.exists() {
        return Err("Input path does not exist".to_string());
    }

    let files = path::get_target_files(&input_path);
    let pkg_files: Vec<_> = files.into_iter().filter(|f| f.extension().map_or(false, |e| e == "pkg")).collect();

    if pkg_files.is_empty() {
        return Ok(PkgStats { processed_files: 0, extracted_files: 0 });
    }

    let mut extracted_count = 0;
    for file in &pkg_files {
        let file_stem = file.file_stem().unwrap().to_str().unwrap();
        let pkg_output_dir = path::get_unique_output_path(&output_path, file_stem);
        
        if let Err(e) = fs::create_dir_all(&pkg_output_dir) {
            return Err(format!("Failed to create output dir: {}", e));
        }
        match pkg::unpack_pkg(file, &pkg_output_dir) {
            Ok(count) => extracted_count += count,
            Err(e) => return Err(e),
        }
        
        let workshop_path = path::expand_path(&config.wallpaper.workshop_path);
        if workshop_path.exists() {
            let scene_name = path::scene_name_from_pkg_stem(file_stem);
            let raw_source = workshop_path.join(&scene_name);
            if raw_source.exists() && raw_source.is_dir() {
                if let Err(e) = copy_dir_recursive_skip_pkg(&raw_source, &pkg_output_dir) {
                    return Err(format!("Failed to copy resources: {}", e));
                }
            }
        }
    }

    Ok(PkgStats { processed_files: pkg_files.len(), extracted_files: extracted_count })
}


pub fn run_tex(config: &Config) -> Result<TexStats, String> {
    let input_path = path::expand_path(&config.unpack.unpacked_output_path);
    
    if !input_path.exists() {
        return Err("Input path does not exist".to_string());
    }

    let files = path::get_target_files(&input_path);
    let tex_files: Vec<_> = files.into_iter().filter(|f| f.extension().map_or(false, |e| e == "tex")).collect();

    if tex_files.is_empty() {
        return Ok(TexStats { processed_files: 0, converted_files: 0 });
    }

    let mut converted_count = 0;
    for file in &tex_files {
        let scene_root = path::find_project_root(file)
            .or_else(|| file.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| input_path.clone());
        let output_dir = path::resolve_tex_output_dir(
            config.tex.converted_output_path.as_deref(),
            &scene_root,
            Some(file),
            Some(&scene_root)
        );

        if let Err(e) = fs::create_dir_all(&output_dir) {
            return Err(format!("Failed to create output dir: {}", e));
        }

        let file_stem = file.file_stem().unwrap().to_str().unwrap();
        let output_file = output_dir.join(file_stem).with_extension("png");

        if let Err(e) = tex::process_tex(file, &output_file) {
            return Err(format!("Failed to process tex {:?}: {}", file, e));
        }
        converted_count += 1;
    }

    Ok(TexStats { processed_files: tex_files.len(), converted_files: converted_count })
}


pub fn run_auto(config: &Config) -> Result<AutoStats, String> {
    let wp_stats = run_wallpaper(config)?;
    let pkg_stats = run_pkg(config)?;
    let tex_stats = run_tex(config)?;
    
    if config.unpack.clean_pkg_temp {
        cleanup_temp(config);
    }
    if config.unpack.clean_unpacked {
        cleanup_unpacked(config);
    }

    Ok(AutoStats { wallpaper: wp_stats, pkg: pkg_stats, tex: tex_stats })
}


pub fn cleanup_temp(config: &Config) {
    let path = path::expand_path(&config.wallpaper.pkg_temp_path);
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}


pub fn cleanup_unpacked(config: &Config) {
    use std::fs;

    let root = path::expand_path(&config.unpack.unpacked_output_path);
    if !root.exists() {
        return;
    }

    let keep_name = "tex_converted";
    let entries = match fs::read_dir(&root) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut has_keep = false;

    for entry in entries.flatten() {
        let p = entry.path();
        let name_match = p.file_name().and_then(|n| n.to_str());

        if let Some(name) = name_match {
            if name == keep_name {
                has_keep = true;
                continue;
            }
        }

        let _ = fs::remove_dir_all(&p).or_else(|_| fs::remove_file(&p));
    }

    if !has_keep {
        let _ = fs::remove_dir_all(&root);
    }
}


fn copy_dir_recursive_skip_pkg(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive_skip_pkg(&entry.path(), &dest_path)?;
        } else {
            if let Some(ext) = entry.path().extension() {
                if ext.eq_ignore_ascii_case("pkg") {
                    continue;
                }
            }
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}
