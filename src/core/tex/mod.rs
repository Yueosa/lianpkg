pub mod structs;
pub mod reader;
pub mod converter;

use std::path::Path;
use std::fs::File;

pub fn process_tex(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let mut file = match File::open(input_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("Failed to open TEX file: {}", e));
        }
    };

    match reader::read_tex(&mut file) {
        Ok(tex_file) => {
            if let Err(e) = converter::convert_and_save(&tex_file, output_path) {
                return Err(format!("Failed to convert/save TEX: {}", e));
            }
        },
        Err(e) => {
            return Err(format!("Failed to read TEX file: {}", e));
        }
    }
    Ok(())
}
