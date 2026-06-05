use crate::{
    SnapshotResult,
    cargo_toml::{Package, WorkspaceConfig},
    error::SnapshotError,
};
use std::{collections::BTreeSet, fs::read_to_string, path::PathBuf};

use crate::{cargo_toml::CargoToml, walk::get_parent};

/// Represents a Cargo.toml manifest file
#[derive(Clone, Debug)]
pub(crate) struct Manifest {
    pub path: PathBuf,
    pub cargo_toml: CargoToml,
}

impl Manifest {
    /// Parse manifest from a path
    pub(crate) fn load(path: impl Into<PathBuf>) -> SnapshotResult<Self> {
        let path = path.into();
        let content = read_to_string(&path)?;
        let cargo_toml: CargoToml = toml::from_str(&content)?;

        Ok(Self { path, cargo_toml })
    }

    /// Returns true if this manifest defines a workspace
    pub(crate) fn is_workspace(&self) -> bool {
        self.cargo_toml.workspace.is_some()
    }

    /// Returns a sorted list of all dependency names from this manifest
    pub(crate) fn dependencies(&self) -> Vec<String> {
        self.cargo_toml
            .dependencies
            .keys()
            .chain(self.cargo_toml.dev_dependencies.keys())
            .chain(self.cargo_toml.build_dependencies.keys())
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Returns the crate name from package section
    /// Panics: Should only be called on crate manifests (not workspace roots)
    pub(crate) fn crate_name(&self) -> SnapshotResult<&str> {
        self.package().map(|p| p.name.as_str())
    }

    pub(crate) fn workspace_name(&self) -> String {
        get_parent(&self.path)
            .map_or(None, |p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_owned())
    }

    pub(crate) fn package(&self) -> SnapshotResult<&Package> {
        match &self.cargo_toml.package {
            Some(package) => Ok(package),
            None => Err(SnapshotError::NoPackage(self.path())),
        }
    }

    pub(crate) fn workspace(&self) -> SnapshotResult<&WorkspaceConfig> {
        match &self.cargo_toml.workspace {
            Some(workspace) => Ok(workspace),
            None => Err(SnapshotError::NoWorkspace(self.path())),
        }
    }

    pub(crate) fn path(&self) -> String {
        self.path.display().to_string()
    }
}
