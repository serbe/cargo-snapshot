use crate::{
    SnapshotResult,
    cargo_toml::{CargoToml, Package, WorkspaceConfig},
    error::SnapshotError,
    walk::get_parent,
};
use std::{borrow::Cow, collections::BTreeSet, fs::read_to_string, path::PathBuf};

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

    /// Returns the crate name from package section.
    /// Returns an error if called on a workspace-root manifest without [package].
    pub(crate) fn crate_name(&self) -> SnapshotResult<&str> {
        self.package().map(|p| p.name.as_str())
    }

    pub(crate) fn workspace_name(&self) -> Cow<'_, str> {
        get_parent(&self.path)
            .ok()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy())
            .map_or(Cow::Borrowed("workspace"), |name| name)
    }

    pub(crate) fn package(&self) -> SnapshotResult<&Package> {
        match &self.cargo_toml.package {
            Some(package) => Ok(package),
            None => Err(SnapshotError::NoPackage(self.path.display().to_string())),
        }
    }

    pub(crate) fn workspace(&self) -> SnapshotResult<&WorkspaceConfig> {
        match &self.cargo_toml.workspace {
            Some(workspace) => Ok(workspace),
            None => Err(SnapshotError::NoWorkspace(self.path.display().to_string())),
        }
    }
}
