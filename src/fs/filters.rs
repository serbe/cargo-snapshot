use std::path::Path;

use crate::config::RUST_EXTENSION;

/// Check if path contains hidden components (Unix hidden files/dirs)
pub(crate) fn is_hidden(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
}

pub(crate) fn is_rust_file(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == RUST_EXTENSION)
}

/// Check if a file path corresponds to a test file
pub(crate) fn is_test_file(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "tests")
        || path.file_stem().is_some_and(|s| {
            let s = s.to_string_lossy();
            s == "tests" || s.ends_with("_test") || s.starts_with("test_")
        })
}
