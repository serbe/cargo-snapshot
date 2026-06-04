use anyhow::{Context, Result};
use std::env::current_dir;
use std::path::{Path, PathBuf};

// use crate::cargo_toml::{Package, WorkspaceConfig};
use crate::manifest::Manifest;
use crate::metadata::MetadataKind;
use crate::walk::find_nearest_cargo_toml;
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
    pub(crate) fn from_current_dir(no_workspace: bool) -> Result<Self> {
        let current_dir = current_dir()?;
        let cargo_toml_path = find_nearest_cargo_toml(&current_dir)?;
        let project_dir = cargo_toml_path
            .parent()
            .with_context(|| format!("failed to get parent of {}", cargo_toml_path.display()))?;

        Self::discover(project_dir, no_workspace)
    }

    /// Create a new project from a directory
    pub(crate) fn discover(dir: &Path, no_workspace: bool) -> Result<Self> {
        let manifest_path = dir.join("Cargo.toml");
        let manifest = Manifest::load(manifest_path)?;

        if manifest.is_workspace() {
            return Self::from_workspace(manifest);
        }

        if !no_workspace
            && let Some(root) = find_workspace_root(manifest.path.parent().with_context(|| {
                format!(
                    "Cargo.toml has no parent directory: {}",
                    manifest.path.display()
                )
            })?)?
        {
            let root_manifest = Manifest::load(root.join("Cargo.toml"))?;
            return Self::from_workspace(root_manifest);
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
    fn from_workspace(manifest: Manifest) -> Result<Self> {
        let root_dir = manifest
            .path
            .parent()
            .with_context(|| format!("Cargo.toml path has no parent: {}", manifest.path.display()))?
            .to_path_buf();

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

    // /// Returns the package information if this is a single crate
    // pub(crate) fn package(&self) -> Option<&Package> {
    //     match self {
    //         Self::Crate { manifest, .. } => manifest.cargo_toml.package.as_ref(),
    //         Self::Workspace { .. } => None,
    //     }
    // }

    // /// Returns the workspace configuration if this is a workspace
    // pub(crate) fn workspace_config(&self) -> Option<&WorkspaceConfig> {
    //     match self {
    //         Self::Crate { .. } => None,
    //         Self::Workspace { manifest, .. } => manifest.cargo_toml.workspace.as_ref(),
    //     }
    // }

    // /// Returns the workspace name if this is a workspace
    // pub(crate) fn workspace_name(&self) -> Option<String> {
    //     match self {
    //         Self::Crate { .. } => None,
    //         Self::Workspace { manifest, .. } => Some(manifest.workspace_name()),
    //     }
    // }

    /// Returns all dependencies from the manifest
    pub(crate) fn dependencies(&self) -> Vec<String> {
        self.manifest().dependencies()
    }

    pub(crate) fn is_workspace(&self) -> bool {
        matches!(self, Self::Workspace { .. })
    }

    pub(crate) fn metadata_kind(&self) -> Result<MetadataKind<'_>> {
        match self {
            Self::Crate { manifest, .. } => {
                let package = manifest.cargo_toml.package.as_ref().with_context(|| {
                    format!(
                        "crate manifest missing [package] section in {}",
                        manifest.path.display()
                    )
                })?;
                Ok(MetadataKind::Crate { package })
            }
            Self::Workspace { manifest, .. } => {
                let config = manifest.cargo_toml.workspace.as_ref().with_context(|| {
                    format!(
                        "workspace manifest missing [workspace] section in {}",
                        manifest.path.display()
                    )
                })?;
                Ok(MetadataKind::Workspace {
                    config,
                    name: manifest.workspace_name(),
                })
            }
        }
    }
}

/// Finds the workspace root by traversing up parent directories
pub(crate) fn find_workspace_root(crate_dir: &Path) -> Result<Option<PathBuf>> {
    for ancestor in crate_dir.ancestors().skip(1) {
        // skip сам crate_dir
        let cargo_toml = ancestor.join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }

        if Manifest::load(&cargo_toml)?.is_workspace() {
            return Ok(Some(ancestor.to_path_buf()));
        }
    }
    Ok(None)
}
