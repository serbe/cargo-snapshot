use std::{env::current_dir, fs::read_to_string};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: Option<String>,
    pub version: Option<String>,
    pub authors: Vec<String>,
    pub edition: Option<String>,
    #[serde(rename = "rust-version")]
    pub rust_version: Option<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub readme: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    #[serde(rename = "license-file")]
    pub license_file: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub workspace: Option<String>,
    pub build: Option<String>,
    pub links: Vec<String>,
    pub exclude: Option<String>,
    pub include: Option<String>,
    pub publish: Option<String>,
    pub metadata: Option<String>,
    #[serde(rename = "default-run")]
    pub default_run: Option<String>,
    pub autolib: Option<String>,
    pub autobins: Vec<String>,
    pub autoexamples: Vec<String>,
    pub autotests: Vec<String>,
    pub autobenches: Vec<String>,
    pub resolver: Option<String>,
}

// pub struct Metadata {
//     pub members: Vec<String>,
//     pub default_members: Vec<String>,
//     pub package: Option<>,
//     pub exclude: Vec<String>,
//     pub metadata: Option<Metadata>,
//     pub resolver: Option<Resolver>,
//     pub dependencies: ,
//     pub lints: ,
// }

// #[derive(Debug, Deserialize)]
// pub struct CargoToml {
//  pub package: Package,
// [lib] — Library target settings.
// [[bin]] — Binary target settings.
// [[example]] — Example target settings.
// [[test]] — Test target settings.
// [[bench]] — Benchmark target settings.

// [dependencies] — Package library dependencies.
// [dev-dependencies] — Dependencies for examples, tests, and benchmarks.
// [build-dependencies] — Dependencies for build scripts.
// [target] — Platform-specific dependencies.
// [badges] — Badges to display on a registry.
// [features] — Conditional compilation features.
// [lints] — Configure linters for this package.
// [hints] — Provide hints for compiling this package.
// [patch] — Override dependencies.
// [replace] — Override dependencies (deprecated).
// [profile] — Compiler settings and optimizations.
// [workspace] — The workspace definition.
// }

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
