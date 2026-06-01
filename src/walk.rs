use anyhow::Result;
use std::{
    fs::{read_dir, read_to_string},
    path::{Component, Path, PathBuf},
};

use toml::{Value, from_str};

/// Read file content as string
pub fn read_file_content(path: &Path) -> Result<String> {
    Ok(read_to_string(path)?)
}

/// Parse TOML file
pub fn read_toml(path: &Path) -> Result<Value> {
    let content = read_file_content(path)?;
    Ok(from_str(&content)?)
}

/// Normalize a path by resolving `.` and `..` components
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            _ => components.push(component),
        }
    }
    components.iter().collect()
}

/// Get directory name from path
pub fn dir_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(String::from)
}

/// Check if path contains hidden components (Unix hidden files/dirs)
pub fn is_hidden(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
}

/// Recursively collect all `.rs` files from a directory
pub fn collect_source_files(
    dir: &Path,
    include_hidden: bool,
    exclude_filter: &impl Fn(&Path) -> bool,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    collect_source_files_recursive(dir, include_hidden, exclude_filter, &mut files)?;
    files.sort();

    Ok(files)
}

/// Recursive helper for collect_source_files
fn collect_source_files_recursive(
    current_dir: &Path,
    include_hidden: bool,
    exclude_filter: &dyn Fn(&Path) -> bool,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    let entries = read_dir(current_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip hidden files/dirs if not included
        if !include_hidden && is_hidden(&path) {
            continue;
        }

        // Skip excluded paths
        if exclude_filter(&path) {
            continue;
        }

        if path.is_dir() {
            collect_source_files_recursive(&path, include_hidden, exclude_filter, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            files.push(path);
        }
    }

    Ok(())
}
