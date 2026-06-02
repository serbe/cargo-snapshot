use anyhow::{Context, Result};
use std::env::current_dir;
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;
use crate::walk::find_nearest_cargo_toml;
use crate::workspace::WorkspaceMember;

/// Represents a Rust project (crate or workspace)
#[derive(Debug)]
pub(crate) struct Project {
    pub root_dir: PathBuf,
    pub manifest: Manifest,
    pub members: Vec<WorkspaceMember>,
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

        if manifest.is_workspace() {
            return Self::from_workspace(manifest);
        }

        if let Some(root) = find_workspace_root(manifest.path.parent().with_context(|| {
            format!(
                "Cargo.toml has no parent directory: {}",
                manifest.path.display()
            )
        })?)? {
            let root_manifest = Manifest::load(root.join("Cargo.toml"))?;
            return Self::from_workspace(root_manifest);
        }

        Ok(Self {
            root_dir: dir.to_path_buf(),
            manifest,
            members: Vec::new(),
        })
    }

    /// Creates a project from a workspace manifest
    fn from_workspace(manifest: Manifest) -> Result<Self> {
        let root_dir = manifest
            .path
            .parent()
            .with_context(|| format!("Cargo.toml path has no parent: {}", manifest.path.display()))?
            .to_path_buf();

        Ok(Self {
            members: WorkspaceMember::collect(&manifest)?,
            root_dir,
            manifest,
        })
    }

    /// Check if this is a workspace root
    pub(crate) fn is_workspace_root(&self) -> bool {
        self.manifest.is_workspace()
    }

    /// Returns the workspace name
    pub(crate) fn workspace_name(&self) -> String {
        self.manifest.crate_name()
    }
}

/// Finds the workspace root by traversing up parent directories
pub(crate) fn find_workspace_root(crate_dir: &Path) -> Result<Option<PathBuf>> {
    let crate_dir = crate_dir
        .parent()
        .with_context(|| format!("failed to get parent of {}", crate_dir.display()))?;

    for parent in crate_dir.ancestors().skip(1) {
        let cargo_toml = parent.join("Cargo.toml");

        if !cargo_toml.exists() {
            continue;
        }

        let workspace_manifest = Manifest::load(&cargo_toml)?;

        if workspace_manifest.is_workspace() {
            return Ok(Some(parent.to_path_buf()));
        }
    }

    Ok(None)
}
