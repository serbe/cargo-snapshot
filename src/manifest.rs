use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use toml::Value;

use crate::walk::{dir_name, normalize_path, read_toml};

/// Represents a Cargo.toml manifest file
#[derive(Debug, Clone)]
pub struct Manifest {
    pub path: PathBuf,
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub edition: Option<String>,
    pub description: Option<String>,
    pub is_workspace: bool,
    pub members: Vec<String>,
    pub dependencies: Vec<String>,
}

impl Manifest {
    /// Parse manifest from a path
    pub fn from_path(path: &Path) -> Result<Self> {
        let value = read_toml(path)
            .with_context(|| format!("Failed to read manifest: {}", path.display()))?;

        let package = value.get("package");

        Ok(Self {
            path: path.to_path_buf(),
            package_name: extract_string(package, "name"),
            version: extract_string(package, "version"),
            edition: extract_string(package, "edition"),
            description: extract_string(package, "description"),
            is_workspace: value.get("workspace").is_some(),
            members: extract_members(&value),
            dependencies: extract_dependency_names(&value),
        })
    }

    /// Find workspace root for this manifest
    pub fn workspace_root(&self) -> Option<PathBuf> {
        if self.is_workspace {
            return self.path.parent().map(PathBuf::from);
        }

        let crate_dir = self.path.parent()?;
        let mut current = crate_dir;

        while let Some(parent) = current.parent() {
            let parent_manifest = parent.join("Cargo.toml");
            if parent_manifest.exists()
                && let Ok(value) = read_toml(&parent_manifest)
                && is_member_of_workspace(&value, crate_dir, parent)
            {
                return Some(parent.to_path_buf());
            }
            current = parent;
        }

        None
    }

    /// Get crate name (prefer package name, fallback to directory name)
    pub fn crate_name(&self) -> String {
        self.package_name
            .clone()
            .or_else(|| dir_name(self.path.parent().unwrap()))
            .unwrap_or_else(|| "unnamed".to_string())
    }

    /// Write metadata to output
    pub fn write_metadata<W: std::io::Write>(&self, output: &mut W) -> std::io::Result<()> {
        writeln!(output, "// --- Metadata ---")?;

        if let Some(name) = &self.package_name {
            writeln!(output, "// name: {}", name)?;
        }
        if let Some(version) = &self.version {
            writeln!(output, "// version: {}", version)?;
        }
        if let Some(edition) = &self.edition {
            writeln!(output, "// edition: {}", edition)?;
        }
        if let Some(description) = &self.description {
            writeln!(output, "// description: {}", description)?;
        }
        if !self.dependencies.is_empty() {
            writeln!(output, "// dependencies: {}", self.dependencies.join(", "))?;
        }

        writeln!(output)?;
        Ok(())
    }
}

/// Helper to extract string field from TOML table
fn extract_string(table: Option<&Value>, key: &str) -> Option<String> {
    table
        .and_then(|t| t.get(key))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Extract workspace members from TOML
fn extract_members(value: &Value) -> Vec<String> {
    value
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_dependency_names(value: &Value) -> Vec<String> {
    let mut deps = Vec::new();

    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = value.get(section).and_then(|v| v.as_table()) {
            deps.extend(table.keys().map(|k| k.to_string()));
        }
    }

    deps.sort();
    deps.dedup();
    deps
}

fn is_member_of_workspace(workspace_toml: &Value, crate_dir: &Path, workspace_root: &Path) -> bool {
    workspace_toml
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .is_some_and(|members| {
            members.iter().any(|m| {
                m.as_str().is_some_and(|pattern| {
                    let full = workspace_root.join(pattern);
                    glob::Pattern::new(&full.to_string_lossy())
                        .is_ok_and(|p| p.matches_path(&normalize_path(crate_dir)))
                })
            })
        })
}
