use serde::Deserialize;
use toml::{Value, from_str};

use crate::{
    SnapshotResult,
    core::package::{Package, WorkspaceConfig},
    error::SnapshotError,
    fs::path_utils::get_parent,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::read_to_string,
    path::PathBuf,
};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ManifestData {
    pub package: Option<Package>,
    #[serde(default)]
    pub workspace: Option<WorkspaceConfig>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Value>,
    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: BTreeMap<String, Value>,
    #[serde(rename = "build-dependencies", default)]
    pub build_dependencies: BTreeMap<String, Value>,
}

#[derive(Clone, Debug)]
pub(crate) struct Manifest {
    pub path: PathBuf,
    pub data: ManifestData,
}

impl Manifest {
    pub(crate) fn load(path: impl Into<PathBuf>) -> SnapshotResult<Self> {
        let path = path.into();

        let content = read_to_string(&path)?;
        let data = from_str(&content)?;
        let manifest = Self {
            path: path.clone(),
            data,
        };

        Ok(manifest)
    }

    pub(crate) fn is_workspace(&self) -> bool {
        self.data.workspace.is_some()
    }

    pub(crate) fn dependencies(&self) -> Vec<String> {
        self.data
            .dependencies
            .keys()
            .chain(self.data.dev_dependencies.keys())
            .chain(self.data.build_dependencies.keys())
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(crate) fn package_name(&self) -> SnapshotResult<String> {
        self.package()?
            .name
            .as_deref()
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| {
                if self.is_workspace() {
                    get_parent(&self.path)
                        .ok()
                        .and_then(|parent| parent.file_name())
                        .map(|name| name.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| SnapshotError::NoPackage(self.path.display().to_string()))
    }

    pub(crate) fn workspace_name(&self) -> String {
        get_parent(&self.path)
            .ok()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy())
            .map_or("workspace".to_owned(), |name| name.to_string())
    }

    pub(crate) fn package(&self) -> SnapshotResult<&Package> {
        match &self.data.package {
            Some(package) => Ok(package),
            None => Err(SnapshotError::NoPackage(self.path.display().to_string())),
        }
    }
}
