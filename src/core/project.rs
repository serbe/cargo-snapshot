use crate::{
    config::{MANIFEST_FILE, SOURCE_DIR},
    core::{crate_target::CrateInfo, manifest::Manifest, workspace::WorkspaceMember},
    discovery::manifest_finder::locate_manifest,
    error::{SnapshotError, SnapshotResult},
    fs::path_utils::get_parent,
    model::project_kind::ProjectKind,
};

use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub(crate) struct Project {
    pub root_dir: PathBuf,
    pub manifest: Manifest,
    pub targets: Vec<CrateInfo>,
    pub workspace_name: Option<String>,
}

impl Project {
    /// Create a new project from current directory
    pub(crate) fn from_current_dir(no_workspace: bool) -> SnapshotResult<Self> {
        let current_dir = current_dir()?;
        let cargo_toml_path = locate_manifest(&current_dir)?;
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
        let name = manifest.package_name()?;
        if name.is_empty() {
            return Err(SnapshotError::NoPackage(
                manifest.path.display().to_string(),
            ));
        }
        Ok(Self {
            root_dir: dir.to_path_buf(),
            targets: vec![CrateInfo {
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
            .map(|member| CrateInfo {
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

    pub(crate) fn targets(&self) -> &[CrateInfo] {
        &self.targets
    }

    /// Returns all dependencies from the manifest
    pub(crate) fn dependencies(&self) -> Vec<String> {
        self.manifest.dependencies()
    }

    pub(crate) fn is_workspace(&self) -> bool {
        self.workspace_name.is_some()
    }

    pub(crate) fn metadata_kind(&self) -> SnapshotResult<ProjectKind<'_>> {
        if let Some(workspace) = &self.manifest.data.workspace {
            return Ok(ProjectKind::Workspace {
                config: workspace,
                name: self.workspace_name.as_deref().unwrap_or("workspace"),
            });
        }

        Ok(ProjectKind::Crate {
            package: self.manifest.package()?,
        })
    }
}

/// Finds the workspace root by traversing up parent directories
pub(crate) fn find_workspace_root(dir: &Path) -> SnapshotResult<Option<Manifest>> {
    for ancestor in dir.ancestors().skip(1) {
        let manifest_path = ancestor.join(MANIFEST_FILE);
        if !manifest_path.exists() {
            continue;
        }
        let manifest = Manifest::load(&manifest_path)?;
        if manifest.is_workspace() {
            return Ok(Some(manifest));
        }
    }
    Ok(None)
}
