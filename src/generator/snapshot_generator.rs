use std::{
    cmp::Ordering,
    fs::{DirEntry, File},
    io::Write,
    path::Path,
};

use crate::{
    SnapshotResult,
    config::settings::Config,
    core::project::WorkspaceContext,
    fs::{
        filters::{is_hidden, is_rust_file},
        path_utils::read_directory,
    },
    generator::{
        manifests::write_extra_files, sources::write_sources, tree::write_project_structure,
    },
    model::metadata::Metadata,
    renderer::{SnapshotRenderer, create_renderer},
};

/// Main writer for generating snapshot output
pub(crate) struct SnapshotBuilder {
    options: Config,
    renderer: Box<dyn SnapshotRenderer>,
}

impl SnapshotBuilder {
    pub(crate) fn new(options: Config) -> Self {
        let renderer = create_renderer(options.format);
        Self { options, renderer }
    }

    /// Writes the complete snapshot to the output path
    pub(crate) fn write(&self) -> SnapshotResult<()> {
        let project = WorkspaceContext::from_current_dir(self.options.no_workspace)?;
        let mut file = File::create(&self.options.output_path)?;

        self.renderer.render_header(&mut file)?;
        self.write_metadata(&mut file, &project)?;

        write_project_structure(&mut file, &project, &self.options, &*self.renderer)?;
        write_sources(&mut file, &project, &self.options, &*self.renderer)?;
        write_extra_files(&mut file, &project, &self.options, &*self.renderer)?;

        Ok(())
    }

    /// Writes metadata section with package/workspace information
    fn write_metadata(
        &self,
        out: &mut impl Write,
        project: &WorkspaceContext,
    ) -> SnapshotResult<()> {
        let metadata = Metadata {
            kind: project.metadata_kind()?,
            dependencies: project.dependencies(),
        };

        self.renderer.render_metadata(out, &metadata)
    }
}

/// Helper function to read sorted directory entries
pub(crate) fn read_sorted_entries(
    dir: &Path,
    include_hidden: bool,
) -> SnapshotResult<Vec<DirEntry>> {
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
