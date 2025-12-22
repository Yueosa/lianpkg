use std::fs;
use std::path::{Path};
use crate::log;

pub fn extract_wallpapers(search_path: &Path, base_output: &Path, video_output_opt: Option<&Path>) {
    let mp4_output = if let Some(v) = video_output_opt {
        v.to_path_buf()
    } else {
        base_output.to_path_buf()
    };
    let pkg_output = base_output.join("Pkg");

    if let Err(e) = fs::create_dir_all(&mp4_output) {
        log::error(&format!("Failed to create video output dir: {}", e));
        return;
    }
    if let Err(e) = fs::create_dir_all(&pkg_output) {
        log::error(&format!("Failed to create pkg output dir: {}", e));
        return;
    }

    log::title("--- 开始扫描目录 ---");
    log::info(&format!("Search Path: {:?}", search_path));
    log::info(&format!("Video Output: {:?}", mp4_output));
    log::info(&format!("PKG Output: {:?}", pkg_output));
    log::info("------------------------------------------");

    let entries = match fs::read_dir(search_path) {
        Ok(e) => e,
        Err(e) => {
            log::error(&format!("Failed to read search path: {}", e));
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        let mut found_mp4 = false;
        let mut found_pkg = false;

        // Read dir content
        let sub_entries = match fs::read_dir(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mut mp4_files = Vec::new();
        let mut pkg_files = Vec::new();

        for sub_entry in sub_entries {
            let sub_entry = match sub_entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let sub_path = sub_entry.path();
            let ext = sub_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

            if ext == "mp4" {
                mp4_files.push(sub_path);
            } else if ext == "pkg" {
                pkg_files.push(sub_path);
            }
        }

        if !mp4_files.is_empty() {
            found_mp4 = true;
            log::info(&format!("✅ [找到视频] 在目录: {}", dir_name));
            for file in mp4_files {
                let dest = mp4_output.join(file.file_name().unwrap());
                if let Err(e) = fs::copy(&file, &dest) {
                    log::error(&format!("Failed to copy mp4: {}", e));
                }
            }
        }

        if !pkg_files.is_empty() {
            found_pkg = true;
            log::info(&format!("📦 [找到 PKG] 在目录: {}", dir_name));
            for file in pkg_files {
                let file_name = file.file_name().unwrap().to_str().unwrap();
                let new_name = format!("{}_{}", dir_name, file_name);
                let dest = pkg_output.join(new_name);
                if let Err(e) = fs::copy(&file, &dest) {
                    log::error(&format!("Failed to copy pkg: {}", e));
                }
            }
        }

        if !found_mp4 && !found_pkg {
            log::info(&format!("❌ [目录无有效素材]: {}", dir_name));
        }
    }
    log::info("------------------------------------------");
    log::info("扫描完成！");
}
