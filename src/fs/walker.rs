use crate::{
    SnapshotResult,
    config::settings::Config,
    fs::filters::{is_hidden, is_rust_file},
    generator::snapshot_generator::read_sorted_entries,
    renderer::SnapshotRenderer,
};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

/// Recursively collect all `.rs` files from a directory
pub(crate) fn collect_rust_files(dir: impl AsRef<Path>, options: &Config) -> Vec<PathBuf> {
    let mut source_files = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| options.include_hidden || !is_hidden(entry.path()))
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(|path| is_rust_file(path))
        .filter(|path| !options.should_exclude(path))
        .collect::<Vec<_>>();

    source_files.sort();

    source_files
}

/// Prints directory tree recursively
pub(crate) fn print_dir_tree(
    out: &mut impl Write,
    dir: &Path,
    prefix: &mut String,
    options: &Config,
    renderer: &dyn SnapshotRenderer,
) -> SnapshotResult<()> {
    let pfx = renderer.tree_prefix();
    let entries = read_sorted_entries(dir, options.include_hidden)?;

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();

        if path.is_dir() {
            writeln!(out, "{pfx}{prefix}{connector}{name}/")?;

            // Extend prefix for children
            let old_len = prefix.len();
            if is_last {
                prefix.push_str("    ");
            } else {
                prefix.push_str("│   ");
            }

            print_dir_tree(out, &path, prefix, options, renderer)?;

            // Restore prefix
            prefix.truncate(old_len);
        } else {
            writeln!(out, "{pfx}{prefix}{connector}{name}")?;
        }
    }

    Ok(())
}
