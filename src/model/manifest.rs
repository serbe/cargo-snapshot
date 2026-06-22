use serde::Deserialize;
use toml::Value;

use crate::{
    SnapshotResult,
    error::SnapshotError,
    fs::walk::get_parent,
    model::cargo_manifest::{CargoManifest, Package, WorkspaceManifest},
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs::read_to_string,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

static MANIFEST_CACHE: LazyLock<Mutex<HashMap<PathBuf, Manifest>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// #[derive(Clone, Debug, Deserialize)]
// pub(crate) struct RawManifest {
//     pub package: Option<Package>,
//     #[serde(default)]
//     pub workspace: Option<WorkspaceManifest>,
//     #[serde(default)]
//     pub dependencies: BTreeMap<String, Value>,
//     #[serde(rename = "dev-dependencies", default)]
//     pub dev_dependencies: BTreeMap<String, Value>,
//     #[serde(rename = "build-dependencies", default)]
//     pub build_dependencies: BTreeMap<String, Value>,
// }

#[derive(Clone, Debug)]
pub(crate) struct Manifest {
    pub path: PathBuf,
    pub cargo_manifest: CargoManifest,
}

impl Manifest {
    pub(crate) fn load(path: impl Into<PathBuf>) -> SnapshotResult<Self> {
        let path = path.into();

        if let Some(manifest) = MANIFEST_CACHE.lock()?.get(&path).cloned() {
            return Ok(manifest);
        }

        let content = read_to_string(&path)?;
        let cargo_manifest: CargoManifest = toml::from_str(&content)?;

        let manifest = Self {
            path: path.clone(),
            cargo_manifest,
        };

        MANIFEST_CACHE.lock()?.insert(path, manifest.clone());

        Ok(manifest)
    }

    pub(crate) fn is_workspace(&self) -> bool {
        self.cargo_manifest.workspace.is_some()
    }

    pub(crate) fn dependencies(&self) -> Vec<String> {
        self.cargo_manifest
            .dependencies
            .keys()
            .chain(self.cargo_manifest.dev_dependencies.keys())
            .chain(self.cargo_manifest.build_dependencies.keys())
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(crate) fn crate_name(&self) -> SnapshotResult<String> {
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
        match &self.cargo_manifest.package {
            Some(package) => Ok(package),
            None => Err(SnapshotError::NoPackage(self.path.display().to_string())),
        }
    }
}
