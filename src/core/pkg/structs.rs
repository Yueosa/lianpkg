#[derive(Debug, Clone)]
pub struct PkgEntry {
    pub name: String,
    pub offset: u32,
    pub size: u32,
}
