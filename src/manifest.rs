use anyhow::{Context, Result};
use std::{collections::BTreeSet, fs::read_to_string, path::PathBuf};

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
    pub(crate) fn crate_name(&self) -> Result<&str> {
        self.cargo_toml
            .package
            .as_ref()
            .map(|p| p.name.as_str())
            .with_context(|| {
                format!(
                    "crate manifest missing [package] section or package.name in {}",
                    self.path.display()
                )
            })
    }

    pub(crate) fn workspace_name(&self) -> String {
        self.path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_owned())
    }
}
