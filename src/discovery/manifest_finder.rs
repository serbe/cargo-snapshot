use std::path::{Path, PathBuf};

use crate::{SnapshotResult, config::MANIFEST_FILE, error::SnapshotError};

/// Finds the nearest Cargo.toml by traversing up parent directories
pub(crate) fn locate_manifest(start_dir: &Path) -> SnapshotResult<PathBuf> {
    let start_dir = start_dir.canonicalize()?;

    for ancestor in start_dir.ancestors() {
        let manifest_path = ancestor.join(MANIFEST_FILE);
        if manifest_path.exists() {
            return Ok(manifest_path);
        }
    }

    Err(SnapshotError::NoCargo(start_dir.display().to_string()))
}
