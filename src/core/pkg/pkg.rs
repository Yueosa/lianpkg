use std::fs;
use std::path::Path;

use super::structs::PkgEntry;
use super::utl::Reader;

pub fn unpack_pkg(file_path: &Path, output_base: &Path) -> Result<usize, String> {
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            return Err(format!("Failed to read file {:?}: {}", file_path, e));
        }
    };

    let mut r = Reader::new(&data);

    let _version = r.read_string();
    let file_count = r.read_u32();

    let mut entries = Vec::with_capacity(file_count as usize);
    for _ in 0..file_count {
        let name = r.read_string();
        let offset = r.read_u32();
        let size = r.read_u32();
        entries.push(PkgEntry { name, offset, size });
    }

    let data_start = r.position();
    let mut extracted_count = 0;

    for entry in entries {
        let start = data_start + entry.offset as usize;
        let end = start + entry.size as usize;

        if end > data.len() {
            return Err(format!("Error: File {} entry out of bounds", entry.name));
        }

        let content = &data[start..end];
        let output_path = output_base.join(&entry.name);

        if let Some(parent) = output_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(format!("Failed to create directory {:?}: {}", parent, e));
            }
        }

        match fs::write(&output_path, content) {
            Ok(_) => {
                extracted_count += 1;
            }
            Err(e) => {
                return Err(format!("Failed to write file {:?}: {}", output_path, e));
            }
        }
    }

    Ok(extracted_count)
}
