use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::walk::is_hidden;
use crate::workspace::WorkspaceMember;

/// Write ASCII tree representation of project structure
pub fn write_project_tree<W: Write>(
    output: &mut W,
    is_workspace_root: bool,
    workspace_members: &[WorkspaceMember],
    current_dir: &Path,
    manifest_path: Option<&Path>,
    include_hidden: bool,
) -> Result<()> {
    writeln!(output, "// ========== PROJECT STRUCTURE ==========")?;
    writeln!(output, "//")?;

    if is_workspace_root {
        let dir_name = current_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        writeln!(output, "// {}/", dir_name)?;

        for (i, member) in workspace_members.iter().enumerate() {
            let connector = if i == workspace_members.len() - 1 {
                "└──"
            } else {
                "├──"
            };
            writeln!(
                output,
                "// {} {}/",
                connector,
                member.relative_path.display()
            )?;
            write_src_tree(output, &member.src_dir(), "│   ", include_hidden)?;
        }
    } else if let Some(manifest) = manifest_path {
        let name = manifest
            .parent()
            .and_then(|p| p.file_name())
            .unwrap_or_default()
            .to_string_lossy();
        writeln!(output, "// {}/", name)?;
        write_src_tree(output, &current_dir.join("src"), "", include_hidden)?;
    }

    writeln!(output, "//")?;
    writeln!(output)?;
    Ok(())
}

/// Write source tree recursively with ASCII art
fn write_src_tree<W: Write>(
    output: &mut W,
    dir: &Path,
    prefix: &str,
    include_hidden: bool,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| include_hidden || !is_hidden(&e.path()))
        .collect();

    entries.sort_by_key(|e| e.path());

    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└──" } else { "├──" };
        let name = path.file_name().unwrap().to_string_lossy();

        if path.is_dir() {
            writeln!(output, "// {}{} {}/", prefix, connector, name)?;
            let new_prefix = format!("{}    ", prefix);
            write_src_tree(output, &path, &new_prefix, include_hidden)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            writeln!(output, "// {}{} {}", prefix, connector, name)?;
        }
    }

    Ok(())
}
