use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub(crate) enum SnapshotError {
    #[error("std io error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Failed get metadata: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),

    #[error("output path would overwrite source file: {0}")]
    OutputIsSource(std::path::PathBuf),

    #[error("Failed to get root package")]
    NoRootPackage,
}
