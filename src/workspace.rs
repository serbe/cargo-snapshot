use crate::{MANIFEST_FILE, SOURCE_DIR, SnapshotResult, walk::get_parent};
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;

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

        Manifest::load(&cargo_toml)
            .ok()
            .and_then(|manifest| {
                manifest.crate_name().ok().map(|name| WorkspaceMember {
                    name: name.to_owned(),
                    src_dir: path.join(SOURCE_DIR),
                    absolute_path: path,
                })
            })
            .or_else(|| {
                tracing::warn!(
                    "Failed to load manifest or manifest not have name: {}",
                    cargo_toml.display()
                );
                None
            })
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

    /// Get source directory path
    pub(crate) fn src_dir(&self) -> &Path {
        &self.src_dir
    }
}
