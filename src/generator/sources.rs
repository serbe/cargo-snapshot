use crate::{
    SnapshotResult,
    config::settings::Config,
    core::{crate_target::CrateInfo, project::WorkspaceContext},
    fs::{path_utils::relative_path, walker::collect_rust_files},
    renderer::SnapshotRenderer,
};
use std::{fs::read_to_string, io::Write, path::Path};

/// Writes all Rust source files from the project
pub(crate) fn write_sources(
    out: &mut impl Write,
    project: &WorkspaceContext,
    options: &Config,
    renderer: &dyn SnapshotRenderer,
) -> SnapshotResult<()> {
    if let Some(name) = &project.workspace_name {
        renderer.render_workspace_heading(out, name)?;
    }

    for target in project.targets() {
        write_crate_sources(out, target, &project.root_dir, options, renderer)?;
    }

    Ok(())
}

/// Writes sources for a single crate target
fn write_crate_sources(
    out: &mut impl Write,
    target: &CrateInfo,
    root_dir: &Path,
    options: &Config,
    renderer: &dyn SnapshotRenderer,
) -> SnapshotResult<()> {
    renderer.render_crate_heading(out, &target.name)?;

    for path in collect_rust_files(&target.src_dir, options) {
        write_file(out, root_dir, &path, renderer)?;
    }

    Ok(())
}

/// Writes a single file with its content
pub(crate) fn write_file(
    out: &mut impl Write,
    root_dir: &Path,
    file_path: &Path,
    renderer: &dyn SnapshotRenderer,
) -> SnapshotResult<()> {
    let content = read_to_string(file_path)?;
    let normalized = relative_path(file_path, root_dir);

    renderer.render_file(out, &normalized, &content)?;
    Ok(())
}
