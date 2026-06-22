use crate::{
    constants::{MANIFEST_FILE, SOURCE_DIR},
    error::{SnapshotError, SnapshotResult},
    fs::walk::{find_nearest_cargo_toml, get_parent},
    model::{
        crate_target::CrateTarget, manifest::Manifest, metadata::MetadataKind,
        workspace::WorkspaceMember,
    },
};

use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

pub(crate) struct Project {
    pub root_dir: PathBuf,
    pub manifest: Manifest,
    pub targets: Vec<CrateTarget>,
    pub workspace_name: Option<String>,
}

impl Project {
    /// Create a new project from current directory
    pub(crate) fn from_current_dir(no_workspace: bool) -> SnapshotResult<Self> {
        let current_dir = current_dir()?;
        let cargo_toml_path = find_nearest_cargo_toml(&current_dir)?;
        let project_dir = get_parent(&cargo_toml_path)?;

        Self::discover(project_dir, no_workspace)
    }

    /// Create a new project from a directory
    pub(crate) fn discover(dir: &Path, no_workspace: bool) -> SnapshotResult<Self> {
        let manifest_path = dir.join(MANIFEST_FILE);
        let manifest = Manifest::load(manifest_path)?;

        if manifest.is_workspace() {
            return Self::from_workspace(&manifest);
        }

        if !no_workspace
            && let Some(root_manifest) = find_workspace_root(get_parent(&manifest.path)?)?
        {
            return Self::from_workspace(&root_manifest);
        }

        Self::single_crate(dir, manifest)
    }

    /// Creates a project from a standalone crate
    fn single_crate(dir: &Path, manifest: Manifest) -> SnapshotResult<Self> {
        let name = manifest.crate_name()?;
        if name.is_empty() {
            return Err(SnapshotError::NoPackage(
                manifest.path.display().to_string(),
            ));
        }
        Ok(Self {
            root_dir: dir.to_path_buf(),
            targets: vec![CrateTarget {
                name,
                src_dir: dir.join(SOURCE_DIR),
            }],
            manifest,
            workspace_name: None,
        })
    }

    /// Creates a project from a workspace manifest
    fn from_workspace(manifest: &Manifest) -> SnapshotResult<Self> {
        let root_dir = get_parent(&manifest.path)?.to_path_buf();

        let targets = WorkspaceMember::collect_members(manifest)?
            .into_iter()
            .map(|member| CrateTarget {
                src_dir: member.src_dir(),
                name: member.name,
            })
            .collect();

        Ok(Self {
            root_dir,
            manifest: manifest.clone(),

            targets,

            workspace_name: Some(manifest.workspace_name()),
        })
    }

    pub(crate) fn targets(&self) -> &[CrateTarget] {
        &self.targets
    }

    /// Returns all dependencies from the manifest
    pub(crate) fn dependencies(&self) -> Vec<String> {
        self.manifest.dependencies()
    }

    pub(crate) fn is_workspace(&self) -> bool {
        self.workspace_name.is_some()
    }

    pub(crate) fn metadata_kind(&self) -> SnapshotResult<MetadataKind<'_>> {
        if let Some(workspace) = &self.manifest.cargo_manifest.workspace {
            return Ok(MetadataKind::Workspace {
                config: workspace,
                name: self.workspace_name.as_deref().unwrap_or("workspace"),
            });
        }

        Ok(MetadataKind::Crate {
            package: self.manifest.package()?,
        })
    }
}

/// Finds the workspace root by traversing up parent directories
pub(crate) fn find_workspace_root(dir: &Path) -> SnapshotResult<Option<Manifest>> {
    for ancestor in dir.ancestors().skip(1) {
        let cargo_manifest = ancestor.join(MANIFEST_FILE);
        if !cargo_manifest.exists() {
            continue;
        }
        let manifest = Manifest::load(&cargo_manifest)?;
        if manifest.is_workspace() {
            return Ok(Some(manifest));
        }
    }
    Ok(None)
}
