use crate::{
    SnapshotResult,
    config::SnapshotConfig,
    model::project::Project,
    renderer::{Renderer, create_renderer},
};
use std::{fs::File, io::Write, path::Path};

pub(crate) mod manifests;
pub(crate) mod sources;
pub(crate) mod tree;

/// Main writer for generating snapshot output
pub(crate) struct SnapshotGenerator {
    options: SnapshotConfig,
    renderer: Box<dyn Renderer>,
}

impl SnapshotGenerator {
    pub(crate) fn new(options: SnapshotConfig) -> Self {
        let renderer = create_renderer(options.format);
        Self { options, renderer }
    }

    /// Writes the complete snapshot to the output path
    pub(crate) fn write(&self, project: &Project, output: &Path) -> SnapshotResult<()> {
        let mut file = File::create(output)?;

        self.renderer.render_header(&mut file)?;
        self.write_metadata(&mut file, project)?;

        // Используем модули для разных частей
        tree::write_project_structure(&mut file, project, &self.options, &*self.renderer)?;
        sources::write_sources(&mut file, project, &self.options, &*self.renderer)?;
        manifests::write_extra_files(&mut file, project, &self.options, &*self.renderer)?;

        Ok(())
    }

    /// Writes metadata section with package/workspace information
    fn write_metadata(&self, out: &mut impl Write, project: &Project) -> SnapshotResult<()> {
        use crate::model::metadata::Metadata;

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
) -> SnapshotResult<Vec<std::fs::DirEntry>> {
    use crate::fs::walk::{is_hidden, is_rust_file, read_directory};
    use std::cmp::Ordering;

    let mut entries: Vec<std::fs::DirEntry> = read_directory(dir)?
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
