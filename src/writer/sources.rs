use crate::{
    SnapshotResult,
    config::SnapshotOptions,
    fs::walk::{collect_rust_sources, relative_display_path},
    model::project::Project,
    renderer::Renderer,
};
use std::{fs::read_to_string, io::Write, path::Path};

/// Writes all Rust source files from the project
pub(crate) fn write_sources(
    out: &mut impl Write,
    project: &Project,
    options: &SnapshotOptions,
    renderer: &dyn Renderer,
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
    target: &crate::model::crate_target::CrateTarget,
    root_dir: &Path,
    options: &SnapshotOptions,
    renderer: &dyn Renderer,
) -> SnapshotResult<()> {
    renderer.render_crate_heading(out, &target.name)?;

    for path in collect_rust_sources(&target.src_dir, options) {
        write_file(out, root_dir, &path, renderer)?;
    }

    Ok(())
}

/// Writes a single file with its content
pub(crate) fn write_file(
    out: &mut impl Write,
    root_dir: &Path,
    file_path: &Path,
    renderer: &dyn Renderer,
) -> SnapshotResult<()> {
    let content = read_to_string(file_path)?;
    let normalized = relative_display_path(file_path, root_dir);

    renderer.render_file(out, &normalized, &content)?;
    Ok(())
}
