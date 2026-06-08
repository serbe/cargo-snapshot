use crate::MANIFEST_FILE;
use crate::error::SnapshotResult;
use std::env::current_dir;
use std::path::{Path, PathBuf};

// use crate::cargo_toml::{Package, WorkspaceConfig};
use crate::manifest::Manifest;
use crate::metadata::MetadataKind;
use crate::walk::{find_nearest_cargo_toml, get_parent};
use crate::workspace::WorkspaceMember;

/// Represents a Rust project (either a standalone crate or a workspace)
#[derive(Debug)]
pub(crate) enum Project {
    /// A standalone crate
    Crate {
        root_dir: PathBuf,
        manifest: Manifest,
    },
    /// A workspace with multiple members
    Workspace {
        root_dir: PathBuf,
        manifest: Manifest,
        members: Vec<WorkspaceMember>,
    },
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
            return Self::from_workspace(manifest);
        }

        if !no_workspace && let Some(root) = find_workspace_root(get_parent(&manifest.path)?)? {
            let root_manifest = Manifest::load(root.join(MANIFEST_FILE))?;
            if root_manifest.is_workspace() {
                return Self::from_workspace(root_manifest);
            }
        }

        Ok(Self::single_crate(dir, manifest))
    }

    /// Creates a project from a standalone crate
    fn single_crate(dir: &Path, manifest: Manifest) -> Self {
        Self::Crate {
            root_dir: dir.to_path_buf(),
            manifest,
        }
    }

    /// Creates a project from a workspace manifest
    fn from_workspace(manifest: Manifest) -> SnapshotResult<Self> {
        let root_dir = get_parent(&manifest.path)?.to_path_buf();
        let members = WorkspaceMember::collect(&manifest)?;

        Ok(Self::Workspace {
            root_dir,
            manifest,
            members,
        })
    }

    /// Returns the root directory of the project
    pub(crate) fn root_dir(&self) -> &Path {
        match self {
            Self::Crate { root_dir, .. } => root_dir,
            Self::Workspace { root_dir, .. } => root_dir,
        }
    }

    /// Returns the manifest of the project
    pub(crate) fn manifest(&self) -> &Manifest {
        match self {
            Self::Crate { manifest, .. } => manifest,
            Self::Workspace { manifest, .. } => manifest,
        }
    }

    /// Returns workspace members if this is a workspace
    pub(crate) fn members(&self) -> Option<&[WorkspaceMember]> {
        match self {
            Self::Workspace { members, .. } => Some(members),
            Self::Crate { .. } => None,
        }
    }

    /// Returns all dependencies from the manifest
    pub(crate) fn dependencies(&self) -> Vec<String> {
        self.manifest().dependencies()
    }

    pub(crate) fn is_workspace(&self) -> bool {
        matches!(self, Self::Workspace { .. })
    }

    pub(crate) fn metadata_kind(&self) -> SnapshotResult<MetadataKind<'_>> {
        match self {
            Self::Crate { manifest, .. } => Ok(MetadataKind::Crate {
                package: manifest.package()?,
            }),
            Self::Workspace { manifest, .. } => Ok(MetadataKind::Workspace {
                config: manifest.workspace()?,
                name: manifest.workspace_name(),
            }),
        }
    }
}

/// Finds the workspace root by traversing up parent directories
pub(crate) fn find_workspace_root(dir: &Path) -> SnapshotResult<Option<PathBuf>> {
    for ancestor in dir.ancestors().skip(1) {
        let cargo_toml = ancestor.join(MANIFEST_FILE);
        if !cargo_toml.exists() {
            continue;
        }

        if Manifest::load(&cargo_toml)?.is_workspace() {
            return Ok(Some(ancestor.to_path_buf()));
        }
    }
    Ok(None)
}
