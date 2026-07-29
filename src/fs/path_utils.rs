use std::{
    fs::{DirEntry, read_dir},
    path::Path,
};

use crate::{SnapshotResult, error::SnapshotError};

pub(crate) fn relative_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map_or(path, |strip_path| strip_path)
        .to_str()
        .map_or("", |str| str)
        .replace('\\', "/")
}

pub(crate) fn get_parent(path: &Path) -> SnapshotResult<&Path> {
    path.parent()
        .ok_or(SnapshotError::NoParent(path.display().to_string()))
}

pub(crate) fn read_directory(path: &Path) -> SnapshotResult<Vec<DirEntry>> {
    let to_err = |e| SnapshotError::ReadDirectory {
        path: path.to_path_buf(),
        source: e,
    };
    read_dir(path)
        .map_err(to_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(to_err)
}
