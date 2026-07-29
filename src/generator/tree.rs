use crate::config::settings::Config;
use crate::core::project::WorkspaceContext;
use crate::fs::path_utils::relative_path;
use crate::fs::walker::print_dir_tree;
use crate::{SnapshotResult, config::SOURCE_DIR, renderer::SnapshotRenderer};
use std::{ffi::OsStr, io::Write};

/// Writes the project structure tree
pub(crate) fn write_project_structure(
    out: &mut impl Write,
    project: &WorkspaceContext,
    options: &Config,
    renderer: &dyn SnapshotRenderer,
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
    project: &WorkspaceContext,
    options: &Config,
    renderer: &dyn SnapshotRenderer,
) -> SnapshotResult<()> {
    let root_name = project
        .root_dir
        .file_name()
        .map_or(OsStr::new("workspace"), |name| name)
        .to_string_lossy();

    renderer.render_structure_root(out, &root_name)?;

    for target in project.targets() {
        let normalize = relative_path(&target.src_dir, &project.root_dir);
        renderer.render_structure_member(out, &normalize)?;

        let mut prefix = String::from("│   ");
        print_dir_tree(out, &target.src_dir, &mut prefix, options, renderer)?;
    }

    Ok(())
}

/// Writes tree for a single crate
fn write_crate_tree(
    out: &mut impl Write,
    project: &WorkspaceContext,
    options: &Config,
    renderer: &dyn SnapshotRenderer,
) -> SnapshotResult<()> {
    let package_name = project.manifest.package_name()?;
    renderer.render_structure_root(out, &package_name)?;

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
