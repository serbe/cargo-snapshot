use anyhow::Result;
use glob::glob;
use std::path::PathBuf;
use tracing::warn;

use crate::manifest::Manifest;

/// A member of a Rust workspace
#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    /// Member name (crate name)
    pub name: String,
    /// Path relative to workspace root
    pub relative_path: PathBuf,
    /// Absolute path to member directory
    pub absolute_path: PathBuf,
    /// Member's manifest
    pub manifest: Manifest,
}

impl WorkspaceMember {
    /// Collect all workspace members from a workspace manifest
    pub fn collect(workspace_manifest: &Manifest) -> Result<Vec<Self>> {
        if !workspace_manifest.is_workspace {
            return Ok(Vec::new());
        }

        let workspace_dir = workspace_manifest.path.parent().unwrap();
        let patterns = &workspace_manifest.members;

        let mut members = Vec::new();

        for pattern in patterns {
            let full_pattern = workspace_dir.join(pattern);
            let full_pattern_str = full_pattern.to_string_lossy();

            let paths = match glob(&full_pattern_str) {
                Ok(paths) => paths.filter_map(|p| p.ok()).collect::<Vec<_>>(),
                Err(e) => {
                    warn!("Invalid glob pattern '{}': {}", pattern, e);
                    continue;
                }
            };

            for absolute_path in paths {
                let cargo_toml = absolute_path.join("Cargo.toml");
                if !cargo_toml.exists() {
                    continue;
                }

                let manifest = match Manifest::from_path(&cargo_toml) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!(
                            "Failed to parse manifest at {}: {}",
                            cargo_toml.display(),
                            e
                        );
                        continue;
                    }
                };

                let relative_path = absolute_path
                    .strip_prefix(workspace_dir)
                    .unwrap_or(&absolute_path)
                    .to_path_buf();

                let name = manifest.crate_name();

                members.push(Self {
                    name,
                    relative_path,
                    absolute_path,
                    manifest,
                });
            }
        }

        // Deduplicate by absolute path
        members.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path));
        members.dedup_by(|a, b| a.absolute_path == b.absolute_path);

        // Sort by name for consistent output
        members.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(members)
    }

    /// Get source directory path
    pub fn src_dir(&self) -> PathBuf {
        self.absolute_path.join("src")
    }
}
