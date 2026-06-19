use crate::writer::read_sorted_entries;
use crate::{
    SnapshotResult, config::SnapshotOptions, constants::SOURCE_DIR,
    fs::walk::relative_display_path, model::project::Project, renderer::Renderer,
};
use std::{ffi::OsStr, io::Write, path::Path};

/// Writes the project structure tree
pub(crate) fn write_project_structure(
    out: &mut impl Write,
    project: &Project,
    options: &SnapshotOptions,
    renderer: &dyn Renderer,
) -> SnapshotResult<()> {
    renderer.render_structure_begin(out)?;

    if project.is_workspace() {
        write_workspace_tree(out, project, options, renderer)?;
    } else {
        write_crate_tree(out, project, options, renderer)?;
    }

    renderer.render_structure_end(out)?;
    Ok(())
}

/// Writes tree for a workspace
fn write_workspace_tree(
    out: &mut impl Write,
    project: &Project,
    options: &SnapshotOptions,
    renderer: &dyn Renderer,
) -> SnapshotResult<()> {
    let root_name = project
        .root_dir
        .file_name()
        .map_or(OsStr::new("workspace"), |name| name)
        .to_string_lossy();

    renderer.render_structure_root(out, &root_name)?;

    for target in project.targets() {
        let normalize = relative_display_path(&target.src_dir, &project.root_dir);
        renderer.render_structure_member(out, &normalize)?;

        let mut prefix = String::from("│   ");
        print_dir_tree(out, &target.src_dir, &mut prefix, options, renderer)?;
    }

    Ok(())
}

/// Writes tree for a single crate
fn write_crate_tree(
    out: &mut impl Write,
    project: &Project,
    options: &SnapshotOptions,
    renderer: &dyn Renderer,
) -> SnapshotResult<()> {
    let crate_name = project.manifest.crate_name()?;
    renderer.render_structure_root(out, &crate_name)?;

    writeln!(out, "{}└── {SOURCE_DIR}/", renderer.tree_prefix())?;

    let mut prefix = String::from("    ");
    print_dir_tree(
        out,
        &project.root_dir.join(SOURCE_DIR),
        &mut prefix,
        options,
        renderer,
    )?;

    Ok(())
}

/// Prints directory tree recursively
pub(crate) fn print_dir_tree(
    out: &mut impl Write,
    dir: &Path,
    prefix: &mut String,
    options: &SnapshotOptions,
    renderer: &dyn Renderer,
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
