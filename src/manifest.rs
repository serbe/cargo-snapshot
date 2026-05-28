use std::{
    collections::HashMap,
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CargoToml {
    package: Option<Package>,

    workspace: Option<Workspace>,

    dependencies: Option<HashMap<String, Dependency>>,

    #[serde(rename = "dev-dependencies")]
    dev_dependencies: Option<HashMap<String, Dependency>>,

    #[serde(rename = "build-dependencies")]
    build_dependencies: Option<HashMap<String, Dependency>>,

    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    edition: String,

    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Workspace {
    members: Option<Vec<String>>,
    exclude: Option<Vec<String>>,

    dependencies: Option<HashMap<String, Dependency>>,

    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Dependency {
    Simple(String),

    Detailed(DetailedDependency),
}

#[derive(Debug, Serialize, Deserialize)]
struct DetailedDependency {
    version: Option<String>,
    path: Option<String>,
    git: Option<String>,
    branch: Option<String>,
    features: Option<Vec<String>>,
    optional: Option<bool>,
    workspace: Option<bool>,

    #[serde(rename = "default-features")]
    default_features: Option<bool>,

    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

impl CargoToml {
    pub fn from_file(path: &Path) -> Result<Self> {
        let manifest_path = path.join("Cargo.toml");
        let content = fs::read_to_string(manifest_path)?;
        let cargo: CargoToml = toml::from_str(&content)?;
        Ok(cargo)
    }
}

// pub fn read_manifest() -> Result<()> {
//     let current_dir = current_dir()?;

//     let mut path = current_dir.canonicalize().with_context(|| {
//         format!(
//             "Failed to canonicalize current directory: {}",
//             current_dir.display()
//         )
//     })?;

//     loop {
//         let manifest_path = path.join("Cargo.toml");

//         if manifest_path.exists() {
//             let manifest = toml::from_str(&read_to_string(path)?)?;
//             if manifest.root_package().is_some() {
//                 return Ok(manifest);
//             }
//         }

//         // Поднимаемся на уровень выше
//         if !path.pop() {
//             break;
//         }
//     }

//     bail!(
//         "Could not find Cargo.toml in current or parent directories.\n\
//         Make sure you're inside a Rust crate directory."
//     )
// }
