use std::fs;
use std::path::Path;

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn read_u32(&mut self) -> u32 {
        if self.pos + 4 > self.buf.len() {
            return 0;
        }
        let v = u32::from_le_bytes(
            self.buf[self.pos..self.pos + 4]
                .try_into()
                .unwrap(),
        );
        self.pos += 4;
        v
    }

    fn read_string(&mut self) -> String {
        let len = self.read_u32() as usize;
        if self.pos + len > self.buf.len() {
            return String::new();
        }
        let s = String::from_utf8(
            self.buf[self.pos..self.pos + len].to_vec()
        ).unwrap_or_else(|_| "<invalid utf8>".to_string());
        self.pos += len;
        s
    }
}

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

    let mut entries = Vec::new();
    for _ in 0..file_count {
        let name = r.read_string();
        let offset = r.read_u32();
        let size = r.read_u32();
        entries.push((name, offset, size));
    }

    let data_start = r.pos;
    let mut extracted_count = 0;

    for (name, offset, size) in entries {
        let start = data_start + offset as usize;
        let end = start + size as usize;
        
        if end > data.len() {
            return Err(format!("Error: File {} entry out of bounds", name));
        }

        let content = &data[start..end];

        let output_path = output_base.join(&name);
        
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
