use std::{io::Write, path::Path};

use anyhow::Result;

use crate::{config::SnapshotOptions, walk::is_hidden};

/// Универсальная функция для печати структуры директорий
pub(crate) fn print_directory_tree<W: Write>(
    output: &mut W,
    dir: &Path,
    prefix: &str,
    line_prefix: &str,
    options: &SnapshotOptions,
) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| {
            let path = e.path();
            let is_rs = path.extension().is_some_and(|ext| ext == "rs");
            let is_dir = path.is_dir();

            if !options.include_hidden && is_hidden(&path) {
                return false;
            }

            is_rs || is_dir
        })
        .collect();

    entries.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if entry.path().is_dir() {
            writeln!(
                output,
                "{}{}{}{}/",
                line_prefix, prefix, connector, name_str
            )?;

            let new_prefix = if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };

            print_directory_tree(output, &entry.path(), &new_prefix, line_prefix, options)?;
        } else {
            writeln!(output, "{}{}{}{}", line_prefix, prefix, connector, name_str)?;
        }
    }
    Ok(())
}
