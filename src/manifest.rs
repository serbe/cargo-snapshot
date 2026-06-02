use anyhow::{Context, Result};
use std::{fs::read_to_string, path::PathBuf};

use crate::cargo_toml::CargoToml;

/// Represents a Cargo.toml manifest file
#[derive(Clone, Debug)]
pub(crate) struct Manifest {
    pub path: PathBuf,
    pub cargo_toml: CargoToml,
}

impl Manifest {
    /// Parse manifest from a path
    pub(crate) fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        let content =
            read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

        let cargo_toml: CargoToml = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        Ok(Self { path, cargo_toml })
    }

    /// Returns true if this manifest defines a workspace
    pub(crate) fn is_workspace(&self) -> bool {
        self.cargo_toml.workspace.is_some()
    }

    /// Returns a sorted list of all dependency names from this manifest
    pub(crate) fn dependencies(&self) -> Vec<String> {
        let mut deps = Vec::new();

        deps.extend(self.cargo_toml.dependencies.keys().cloned());
        deps.extend(self.cargo_toml.dev_dependencies.keys().cloned());
        deps.extend(self.cargo_toml.build_dependencies.keys().cloned());
        deps.sort();
        deps.dedup();

        deps
    }

    /// Returns the crate name from package section or falls back to directory name
    pub(crate) fn crate_name(&self) -> String {
        self.cargo_toml
            .package
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| {
                self.path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "unnamed".into())
            })
    }
}
