use crate::{
    SnapshotResult, config::SnapshotConfig, fs::walk::relative_path, model::project::Project,
    renderer::Renderer,
};
use std::{fs::read_to_string, io::Write, path::Path};

/// Writes additional files like Cargo.toml and README.md
pub(crate) fn write_extra_files(
    out: &mut impl Write,
    project: &Project,
    options: &SnapshotConfig,
    _renderer: &dyn Renderer,
) -> SnapshotResult<()> {
    // Per-crate manifests
    if options.include_cargo_toml {
        for target in project.targets() {
            let Some(crate_root) = target.src_dir.parent() else {
                continue;
            };
            let manifest_path = crate_root.join("Cargo.toml");
            write_manifest(
                out,
                &manifest_path,
                &format!("Cargo.toml for {}", target.name),
                project,
            )?;
        }
    }

    // Workspace manifest
    if options.include_workspace_toml {
        let ws_manifest = project.root_dir.join("Cargo.toml");
        write_manifest(out, &ws_manifest, "Workspace Cargo.toml", project)?;
    }

    // README.md
    if options.include_readme {
        write_readme(out, project)?;
    }

    Ok(())
}

/// Writes a single manifest file (Cargo.toml)
fn write_manifest(
    out: &mut impl Write,
    path: &Path,
    label: &str,
    project: &Project,
) -> SnapshotResult<()> {
    if !path.exists() {
        return Ok(());
    }

    let content = read_to_string(path)?;
    let normalize = relative_path(path, &project.root_dir);

    writeln!(out, "\n## {label}: `{normalize}`\n")?;
    writeln!(out, "```toml")?;
    writeln!(out, "{content}")?;
    writeln!(out, "```")?;

    Ok(())
}

/// Writes README.md if it exists
fn write_readme(out: &mut impl Write, project: &Project) -> SnapshotResult<()> {
    let readme = project.root_dir.join("README.md");

    if !readme.exists() {
        return Ok(());
    }

    let content = read_to_string(&readme)?;

    writeln!(out, "\n## README.md\n")?;
    writeln!(out, "```markdown")?;
    writeln!(out, "{content}")?;
    writeln!(out, "```")?;

    Ok(())
}
