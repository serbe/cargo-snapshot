use anyhow::{Context, Result};
use std::{
    fs::{DirEntry, File, read_dir, read_to_string},
    io::Write,
    path::Path,
};

use crate::{
    config::SnapshotOptions,
    metadata::Metadata,
    project::Project,
    renderer::{Renderer, create_renderer},
    walk::{collect_source_files, is_hidden},
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
    pub(crate) fn write(&self, project: &Project, output: &Path) -> Result<()> {
        let mut file = File::create(output)?;

        self.renderer.render_header(&mut file)?;
        self.write_metadata(&mut file, project)?;
        self.write_sources(&mut file, project)?;

        Ok(())
    }

    /// Writes metadata section with package/workspace information
    fn write_metadata(&self, out: &mut impl Write, project: &Project) -> Result<()> {
        let metadata = Metadata {
            kind: project.metadata_kind()?,
            dependencies: project.dependencies(),
        };

        self.renderer.render_metadata(out, &metadata)
    }

    fn write_sources(&self, out: &mut impl Write, project: &Project) -> Result<()> {
        if project.is_workspace() {
            self.write_workspace(out, project)
        } else {
            self.write_single_crate(out, project)
        }
    }

    fn write_workspace(&self, out: &mut impl Write, project: &Project) -> Result<()> {
        let manifest = project.manifest();
        let members = project.members().expect("workspace must have members");

        self.renderer
            .render_workspace_heading(out, &manifest.workspace_name())?;
        self.write_project_structure(out, project)?;

        for member in members {
            self.write_crate_sources(out, &member.name, member.src_dir(), project.root_dir())?;
        }
        Ok(())
    }

    fn write_single_crate(&self, out: &mut impl Write, project: &Project) -> Result<()> {
        let manifest = project.manifest();
        self.write_project_structure(out, project)?; // ← единый метод
        self.write_crate_sources(
            out,
            manifest.crate_name()?,
            &project.root_dir().join("src"),
            project.root_dir(),
        )
    }

    fn write_project_structure(&self, out: &mut impl Write, project: &Project) -> Result<()> {
        let r = &self.renderer;
        r.render_structure_begin(out)?;

        if let Some(members) = project.members() {
            let root_name = project
                .root_dir()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            r.render_structure_root(out, &root_name)?;

            for member in members {
                let relative = member
                    .absolute_path
                    .strip_prefix(project.root_dir())
                    .unwrap_or(&member.absolute_path)
                    .display()
                    .to_string();
                r.render_structure_member(out, &relative)?;

                let mut prefix = String::from("│   ");
                self.print_dir_tree(out, member.src_dir(), &mut prefix)?;
            }
        } else {
            r.render_structure_root(out, project.manifest().crate_name()?)?;
            writeln!(out, "{}└── src/", r.tree_prefix())?;

            let mut prefix = String::from("    ");
            self.print_dir_tree(out, &project.root_dir().join("src"), &mut prefix)?;
        }

        r.render_structure_end(out)?;
        writeln!(out)?;
        Ok(())
    }

    fn write_crate_sources(
        &self,
        out: &mut impl Write,
        crate_name: &str,
        src_dir: &Path,
        root_dir: &Path,
    ) -> Result<()> {
        self.renderer.render_crate_heading(out, crate_name)?;

        for path in collect_source_files(src_dir, &self.options) {
            self.write_file(out, root_dir, &path)?;
        }
        Ok(())
    }

    fn write_file(&self, out: &mut impl Write, root_dir: &Path, file_path: &Path) -> Result<()> {
        let content = read_to_string(file_path)
            .with_context(|| format!("failed to read file: {}", file_path.display()))?;
        let relative = file_path.strip_prefix(root_dir).unwrap_or(file_path);
        let normalized = relative.to_str().unwrap_or("").replace('\\', "/");

        self.renderer.render_file(out, &normalized, &content)?;
        Ok(())
    }

    fn print_dir_tree(&self, out: &mut impl Write, dir: &Path, prefix: &mut String) -> Result<()> {
        let pfx = self.renderer.tree_prefix();
        let entries = read_sorted_entries(dir, self.options.include_hidden)
            .with_context(|| format!("failed to read directory: {}", dir.display()))?;

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
}

fn read_sorted_entries(dir: &Path, include_hidden: bool) -> std::io::Result<Vec<DirEntry>> {
    let mut entries: Vec<DirEntry> = read_dir(dir)?
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .filter(|e| {
            let p = e.path();
            if !include_hidden && is_hidden(&p) {
                return false;
            }
            p.is_dir() || p.extension().is_some_and(|x| x == "rs")
        })
        .collect();

    entries.sort_by(|a, b| {
        let a_path = a.path();
        let b_path = b.path();

        match (a_path.is_dir(), b_path.is_dir()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    Ok(entries)
}
