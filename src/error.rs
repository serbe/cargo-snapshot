use std::{path::PathBuf, sync::PoisonError};

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

    #[error("Failed to read directory {path}: {source}")]
    ReadDirectory {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to acquire lock for manifest cache: {0}")]
    CacheLock(String),
}

impl<T> From<PoisonError<T>> for SnapshotError {
    fn from(err: PoisonError<T>) -> Self {
        SnapshotError::CacheLock(err.to_string())
    }
}

pub(crate) type SnapshotResult<T> = Result<T, SnapshotError>;
