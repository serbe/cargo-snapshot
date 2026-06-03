use anyhow::{Context, Result};
use std::env::current_dir;
use std::path::{Path, PathBuf};

use crate::cargo_toml::{Package, WorkspaceConfig};
use crate::manifest::Manifest;
use crate::walk::find_nearest_cargo_toml;
use crate::workspace::WorkspaceMember;

/// Common fields shared by all project types
#[derive(Debug)]
pub(crate) struct ProjectBase {
    pub root_dir: PathBuf,
    pub manifest: Manifest,
}

impl ProjectBase {
    /// Returns the root directory of the project
    pub(crate) fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Returns the manifest of the project
    pub(crate) fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}

/// Represents a Rust project (either a standalone crate or a workspace)
#[derive(Debug)]
pub(crate) enum Project {
    /// A standalone crate
    Crate(ProjectBase),
    /// A workspace with multiple members
    Workspace {
        base: ProjectBase,
        members: Vec<WorkspaceMember>,
    },
}

impl Project {
    /// Create a new project from current directory
    pub(crate) fn from_current_dir() -> Result<Self> {
        let current_dir = current_dir()?;
        let cargo_toml_path = find_nearest_cargo_toml(&current_dir)?;
        let project_dir = cargo_toml_path
            .parent()
            .with_context(|| format!("failed to get parent of {}", cargo_toml_path.display()))?;

        Self::discover(project_dir)
    }

    /// Create a new project from a directory
    pub(crate) fn discover(dir: &Path) -> Result<Self> {
        let manifest_path = dir.join("Cargo.toml");
        let manifest = Manifest::load(manifest_path)?;

        // Case 1: This is a workspace root
        if manifest.is_workspace() {
            return Self::from_workspace(manifest);
        }

        // Case 2: This is a member of a parent workspace
        if let Some(root) = find_workspace_root(manifest.path.parent().with_context(|| {
            format!(
                "Cargo.toml has no parent directory: {}",
                manifest.path.display()
            )
        })?)? {
            let root_manifest = Manifest::load(root.join("Cargo.toml"))?;
            return Self::from_workspace(root_manifest);
        }

        // Case 3: Standalone crate
        Ok(Self::single_crate(dir, manifest))
    }

    /// Creates a project from a standalone crate
    fn single_crate(dir: &Path, manifest: Manifest) -> Self {
        Self::Crate(ProjectBase {
            root_dir: dir.to_path_buf(),
            manifest,
        })
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
            base: ProjectBase { root_dir, manifest },
            members,
        })
    }

    /// Returns the base fields (root dir and manifest)
    pub(crate) fn base(&self) -> &ProjectBase {
        match self {
            Self::Crate(base) => base,
            Self::Workspace { base, .. } => base,
        }
    }

    /// Returns the root directory of the project
    pub(crate) fn root_dir(&self) -> &Path {
        self.base().root_dir()
    }

    /// Returns the manifest of the project
    pub(crate) fn manifest(&self) -> &Manifest {
        self.base().manifest()
    }

    /// Returns workspace members if this is a workspace
    pub(crate) fn members(&self) -> Option<&[WorkspaceMember]> {
        match self {
            Self::Workspace { members, .. } => Some(members),
            Self::Crate(_) => None,
        }
    }

    /// Returns the package information if this is a single crate
    pub(crate) fn package(&self) -> Option<&Package> {
        match self {
            Project::Crate(base) => base.manifest.cargo_toml.package.as_ref(),
            Project::Workspace { .. } => None,
        }
    }

    /// Returns the workspace configuration if this is a workspace
    pub(crate) fn workspace_config(&self) -> Option<&WorkspaceConfig> {
        match self {
            Project::Crate(_) => None,
            Project::Workspace { base, .. } => base.manifest.cargo_toml.workspace.as_ref(),
        }
    }

    /// Returns the workspace name if this is a workspace
    pub(crate) fn workspace_name(&self) -> Option<String> {
        match self {
            Project::Crate(_) => None,
            Project::Workspace { base, .. } => Some(base.manifest.workspace_name()),
        }
    }

    /// Returns all dependencies from the manifest
    pub(crate) fn dependencies(&self) -> Vec<String> {
        self.manifest().dependencies()
    }

    pub(crate) fn is_workspace(&self) -> bool {
        matches!(self, Project::Workspace { .. })
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
