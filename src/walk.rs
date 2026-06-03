use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
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
        .into_iter()
        .filter_entry(|entry| options.include_hidden || !is_hidden(entry.path()))
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().is_some_and(|e| e == "rs"))
        .filter(|path| !options.should_exclude(path))
        .collect::<Vec<_>>();

    files.sort();

    files
}

/// Finds the nearest Cargo.toml by traversing up parent directories
pub(crate) fn find_nearest_cargo_toml(start_dir: &Path) -> Result<PathBuf> {
    let start_dir = start_dir
        .canonicalize()
        .with_context(|| format!("failed to canonicalize path {}", start_dir.display()))?;

    for ancestor in start_dir.ancestors() {
        let cargo_toml = ancestor.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(cargo_toml);
        }
    }

    anyhow::bail!(
        "Cargo.toml not found in {} or any parent directory",
        start_dir.display()
    );
}

/// Check if a file path corresponds to a test file
pub(crate) fn is_test_file(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "tests")
        || path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "test" || s.ends_with("_test"))
}
