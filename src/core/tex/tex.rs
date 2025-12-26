use std::fs::File;
use std::path::Path;

use super::converter;
use super::reader;

pub fn process_tex(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let mut file = File::open(input_path)
        .map_err(|e| format!("Failed to open TEX file: {}", e))?;

    let tex_file = reader::read_tex(&mut file)
        .map_err(|e| format!("Failed to read TEX file: {}", e))?;

    converter::convert_and_save(&tex_file, output_path)
        .map_err(|e| format!("Failed to convert/save TEX: {}", e))
}
