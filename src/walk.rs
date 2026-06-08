use crate::{MANIFEST_FILE, RUST_EXTENSION, SnapshotResult, error::SnapshotError};
use std::{
    fs::{DirEntry, read_dir},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::config::SnapshotOptions;

/// Check if path contains hidden components (Unix hidden files/dirs)
pub(crate) fn is_hidden(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
}

/// Recursively collect all `.rs` files from a directory
pub(crate) fn collect_source_files(dir: &Path, options: &SnapshotOptions) -> Vec<PathBuf> {
    let mut files = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| options.include_hidden || !is_hidden(entry.path()))
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().is_some_and(|e| e == RUST_EXTENSION))
        .filter(|path| !options.should_exclude(path))
        .collect::<Vec<_>>();

    files.sort();

    files
}

/// Finds the nearest Cargo.toml by traversing up parent directories
pub(crate) fn find_nearest_cargo_toml(start_dir: &Path) -> SnapshotResult<PathBuf> {
    let start_dir = start_dir.canonicalize()?;

    for ancestor in start_dir.ancestors() {
        let cargo_toml = ancestor.join(MANIFEST_FILE);
        if cargo_toml.exists() {
            return Ok(cargo_toml);
        }
    }

    Err(SnapshotError::NoCargo(start_dir.display().to_string()))
}

/// Check if a file path corresponds to a test file
pub(crate) fn is_test_file(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "tests")
        || path.file_stem().is_some_and(|s| s == "tests")
}

pub(crate) fn get_parent(path: &Path) -> SnapshotResult<&Path> {
    path.parent()
        .ok_or(SnapshotError::NoParent(path.display().to_string()))
}

pub(crate) fn read_directory(path: &Path) -> SnapshotResult<Vec<DirEntry>> {
    read_dir(path)
        .map_err(|e| SnapshotError::ReadDirectory {
            path: path.to_path_buf(),
            source: e,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| SnapshotError::ReadDirectory {
            path: path.to_path_buf(),
            source: e,
        })
}
