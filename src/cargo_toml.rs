use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use toml::{Table, Value};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CargoToml {
    pub package: Option<Package>,
    pub workspace: Option<WorkspaceConfig>,

    #[serde(default)]
    pub dependencies: BTreeMap<String, Value>,

    #[serde(rename = "dev-dependencies", default)]
    pub dev_dependencies: BTreeMap<String, Value>,

    #[serde(rename = "build-dependencies", default)]
    pub build_dependencies: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkspaceConfig {
    #[serde(default)]
    pub members: Vec<String>,

    #[serde(default)]
    pub resolver: Option<String>,

    #[serde(default)]
    pub package: Option<WorkspacePackage>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub(crate) struct WorkspacePackage {
    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub edition: Option<String>,

    #[serde(default)]
    pub license: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Package {
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_inheritable_field")]
    pub version: Option<String>,
    #[serde(default, deserialize_with = "deserialize_inheritable_field")]
    pub edition: Option<String>,
    #[serde(default, deserialize_with = "deserialize_inheritable_field")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_inheritable_field")]
    pub license: Option<String>,
}

/// Deserializes an inheritable field that can be either a direct value or `{ workspace = true }`
fn deserialize_inheritable_field<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum InheritableField {
        Value(String),
        #[allow(dead_code)]
        Workspace(Table),
    }

    match InheritableField::deserialize(deserializer)? {
        InheritableField::Value(s) => Ok(Some(s)),
        InheritableField::Workspace(_) => Ok(None),
    }
}
