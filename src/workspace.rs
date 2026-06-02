use anyhow::Result;
use std::path::PathBuf;

use crate::manifest::Manifest;

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceMember {
    pub name: String,
    pub absolute_path: PathBuf,
}

impl WorkspaceMember {
    /// Collects all workspace members by resolving member patterns from workspace manifest
    pub(crate) fn collect(workspace_manifest: &Manifest) -> Result<Vec<Self>> {
        let Some(workspace) = &workspace_manifest.cargo_toml.workspace else {
            return Ok(Vec::new());
        };

        let workspace_root = workspace_manifest
            .path
            .parent()
            .expect("Workspace manifest path has no parent");
        let mut members = Vec::new();

        for pattern in &workspace.members {
            let pattern_path = workspace_root.join(pattern);

            if pattern_path.exists() && pattern_path.is_dir() {
                let cargo_toml = pattern_path.join("Cargo.toml");
                match Manifest::load(&cargo_toml) {
                    Ok(manifest) => {
                        members.push(WorkspaceMember {
                            name: manifest.crate_name(),
                            absolute_path: pattern_path,
                        });
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load manifest at {}: {}",
                            cargo_toml.display(),
                            e
                        );
                    }
                }
            } else {
                let pattern_str = pattern_path.to_string_lossy();
                match glob::glob(&pattern_str) {
                    Ok(paths) => {
                        for path in paths.flatten() {
                            if path.join("Cargo.toml").exists() {
                                match Manifest::load(path.join("Cargo.toml")) {
                                    Ok(manifest) => {
                                        members.push(WorkspaceMember {
                                            name: manifest.crate_name(),
                                            absolute_path: path,
                                        });
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Failed to load manifest at {}: {}",
                                            path.display(),
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Invalid glob pattern '{}': {}", pattern_str, e);
                    }
                }
            }
        }

        members.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(members)
    }

    /// Get source directory path
    pub(crate) fn src_dir(&self) -> PathBuf {
        self.absolute_path.join("src")
    }
}
