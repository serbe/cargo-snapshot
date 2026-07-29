use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct CrateInfo {
    pub name: String,
    pub src_dir: PathBuf,
}
