use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct CrateTarget {
    pub name: String,
    pub src_dir: PathBuf,
}
