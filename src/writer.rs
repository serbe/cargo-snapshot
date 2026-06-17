use crate::{
    SnapshotResult,
    config::SnapshotOptions,
    constants::SOURCE_DIR,
    metadata::Metadata,
    project::Project,
    renderer::{Renderer, create_renderer},
    walk::{collect_source_files, is_hidden, is_rust_file, normalize_path, read_directory},
};
use std::{
    cmp::Ordering,
    ffi::OsStr,
    fs::{DirEntry, File, read_to_string},
    io::Write,
    path::Path,
};

/// Main writer for generating snapshot output
pub(crate) struct SnapshotWriter {
    options: SnapshotOptions,
    renderer: Box<dyn Renderer>,
}

impl SnapshotWriter {
    pub(crate) fn new(options: SnapshotOptions) -> Self {
        let renderer = create_renderer(options.format);
        Self { options, renderer }
    }

    /// Writes the complete snapshot to the output path
    pub(crate) fn write(&self, project: &Project, output: &Path) -> SnapshotResult<()> {
        let mut file = File::create(output)?;

        self.renderer.render_header(&mut file)?;
        self.write_metadata(&mut file, project)?;
        self.write_sources(&mut file, project)?;
        self.write_additional_files(&mut file, project)?;

        Ok(())
    }

    /// Writes metadata section with package/workspace information
    fn write_metadata(&self, out: &mut impl Write, project: &Project) -> SnapshotResult<()> {
        let metadata = Metadata {
            kind: project.metadata_kind()?,
            dependencies: project.dependencies(),
        };

        self.renderer.render_metadata(out, &metadata)
    }

    fn write_sources(&self, out: &mut impl Write, project: &Project) -> SnapshotResult<()> {
        if let Some(name) = project
            .is_workspace()
            .then(|| project.manifest().workspace_name())
        {
            self.renderer.render_workspace_heading(out, &name)?;
        }
        self.write_project_structure(out, project)?;

        for (name, src_dir) in project.crate_targets()? {
            self.write_crate_sources(out, &name, &src_dir, project.root_dir())?;
        }
        Ok(())
    }

    fn write_project_structure(
        &self,
        out: &mut impl Write,
        project: &Project,
    ) -> SnapshotResult<()> {
        let r = &self.renderer;
        r.render_structure_begin(out)?;

        if let Some(members) = project.members() {
            let root_name = project
                .root_dir()
                .file_name()
                .map_or(OsStr::new("workspace"), |name| name)
                .to_string_lossy();
            r.render_structure_root(out, &root_name)?;

            for member in members {
                let normalize = normalize_path(&member.absolute_path, project.root_dir());
                r.render_structure_member(out, &normalize)?;

                let mut prefix = String::from("│   ");
                self.print_dir_tree(out, &member.src_dir, &mut prefix)?;
            }
        } else {
            r.render_structure_root(out, project.manifest().crate_name()?)?;
            writeln!(out, "{}└── {SOURCE_DIR}/", r.tree_prefix())?;

            let mut prefix = String::from("    ");
            self.print_dir_tree(out, &project.root_dir().join(SOURCE_DIR), &mut prefix)?;
        }

        r.render_structure_end(out)?;
        Ok(())
    }

    fn write_crate_sources(
        &self,
        out: &mut impl Write,
        crate_name: &str,
        src_dir: &Path,
        root_dir: &Path,
    ) -> SnapshotResult<()> {
        self.renderer.render_crate_heading(out, crate_name)?;

        for path in collect_source_files(src_dir, &self.options) {
            self.write_file(out, root_dir, &path)?;
        }
        Ok(())
    }

    fn write_manifest(
        &self,
        out: &mut impl Write,
        path: &Path,
        label: &str,
        project: &Project,
    ) -> SnapshotResult<()> {
        if !path.exists() {
            return Ok(());
        }

        let content = read_to_string(path)?;
        let normalize = normalize_path(path, project.root_dir());
        writeln!(out, "\n## {label}: `{normalize}`\n")?;
        writeln!(out, "```toml")?;
        writeln!(out, "{content}")?;
        writeln!(out, "```")?;
        Ok(())
    }

    fn write_file(
        &self,
        out: &mut impl Write,
        root_dir: &Path,
        file_path: &Path,
    ) -> SnapshotResult<()> {
        let content = read_to_string(file_path)?;
        let normalized = normalize_path(file_path, root_dir);

        self.renderer.render_file(out, &normalized, &content)?;
        Ok(())
    }

    fn print_dir_tree(
        &self,
        out: &mut impl Write,
        dir: &Path,
        prefix: &mut String,
    ) -> SnapshotResult<()> {
        let pfx = self.renderer.tree_prefix();
        let entries = read_sorted_entries(dir, self.options.include_hidden)?;

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

                self.print_dir_tree(out, &path, prefix)?;

                // Restore prefix
                prefix.truncate(old_len);
            } else {
                writeln!(out, "{pfx}{prefix}{connector}{name}")?;
            }
        }
        Ok(())
    }

    fn write_additional_files(
        &self,
        out: &mut impl Write,
        project: &Project,
    ) -> SnapshotResult<()> {
        // Per-crate manifests
        if self.options.include_cargo_toml {
            for (name, src_dir) in project.crate_targets()? {
                let Some(crate_root) = src_dir.parent() else {
                    continue;
                };
                let manifest_path = crate_root.join("Cargo.toml");
                self.write_manifest(
                    out,
                    &manifest_path,
                    &format!("Cargo.toml for {name}"),
                    project,
                )?;
            }
        }

        // Workspace manifest
        if self.options.include_workspace_toml {
            let ws_manifest = project.root_dir().join("Cargo.toml");
            self.write_manifest(out, &ws_manifest, "Workspace Cargo.toml", project)?;
        }

        // README.md
        if self.options.include_readme {
            let readme = project.root_dir().join("README.md");
            if readme.exists() {
                let content = read_to_string(&readme)?;
                writeln!(out, "\n## README.md\n")?;
                writeln!(out, "```markdown")?;
                writeln!(out, "{content}")?;
                writeln!(out, "```")?;
            }
        }

        Ok(())
    }
}

fn read_sorted_entries(dir: &Path, include_hidden: bool) -> SnapshotResult<Vec<DirEntry>> {
    let mut entries: Vec<DirEntry> = read_directory(dir)?
        .into_iter()
        .filter(|e| {
            let p = e.path();
            if !include_hidden && is_hidden(&p) {
                return false;
            }
            p.is_dir() || is_rust_file(&p)
        })
        .collect();

    entries.sort_by(|a, b| {
        let a_path = a.path();
        let b_path = b.path();

        match (a_path.is_dir(), b_path.is_dir()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    Ok(entries)
}
