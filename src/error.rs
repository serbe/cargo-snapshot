use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum SnapshotError {
    #[error("std io error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Failed to parse TOML config: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to parse args: {0}")]
    GlobParse(#[from] glob::PatternError),

    #[error("Failed to get parent of {0}")]
    NoParent(String),

    #[error("Cargo.toml not found in {0} or any parent directory")]
    NoCargo(String),

    #[error("Crate manifest missing [package] section or package.name in {0}")]
    NoPackage(String),

    #[error("Workspace manifest missing [workspace] section in {0}")]
    NoWorkspace(String),

    #[error("Failed to read directory {path}: {source}")]
    ReadDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub(crate) type SnapshotResult<T> = Result<T, SnapshotError>;
