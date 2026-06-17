use crate::{
    SnapshotResult,
    constants::{MANIFEST_FILE, SOURCE_DIR},
    manifest::Manifest,
    walk::get_parent,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceMember {
    pub name: String,
    pub src_dir: PathBuf,
    pub absolute_path: PathBuf,
}

impl WorkspaceMember {
    /// Tries to load a workspace member from a directory path
    fn try_load(path: PathBuf) -> Option<Self> {
        let cargo_toml = path.join(MANIFEST_FILE);

        match Manifest::load(&cargo_toml) {
            Ok(manifest) => match manifest.crate_name() {
                Ok(name) => Some(WorkspaceMember {
                    name: name.to_owned(),
                    src_dir: path.join(SOURCE_DIR),
                    absolute_path: path,
                }),
                Err(e) => {
                    tracing::warn!("Member missing package name in {}: {}", path.display(), e);
                    None
                }
            },
            Err(e) => {
                tracing::warn!("Failed to load member manifest {}: {}", path.display(), e);
                None
            }
        }
    }

    /// Expands a single member pattern into a vector of paths
    fn expand_pattern(pattern_path: PathBuf) -> Vec<PathBuf> {
        if pattern_path.exists() && pattern_path.is_dir() {
            return vec![pattern_path];
        }

        let pattern_str = pattern_path.to_string_lossy().to_string();
        match glob::glob(&pattern_str) {
            Ok(paths) => paths.filter_map(Result::ok).collect(),
            Err(e) => {
                tracing::warn!("Invalid glob pattern '{}': {}", pattern_str, e);
                Vec::new()
            }
        }
    }

    /// Collects all workspace members by resolving member patterns from workspace manifest
    pub(crate) fn collect(workspace_manifest: &Manifest) -> SnapshotResult<Vec<Self>> {
        let Some(workspace) = &workspace_manifest.cargo_toml.workspace else {
            return Ok(Vec::new());
        };

        let workspace_root = get_parent(&workspace_manifest.path)?;

        let mut members: Vec<Self> = workspace
            .members
            .iter()
            .flat_map(|pattern| Self::expand_pattern(workspace_root.join(pattern)))
            .filter_map(Self::try_load)
            .collect();

        members.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(members)
    }
}
