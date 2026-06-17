use std::borrow::Cow;

use crate::cargo_toml::{Package, WorkspaceConfig};

/// Represents the kind of metadata (either a crate or a workspace)
pub(crate) enum MetadataKind<'a> {
    Crate {
        package: &'a Package,
    },
    Workspace {
        config: &'a WorkspaceConfig,
        name: Cow<'a, str>,
    },
}

/// Metadata structure containing all information to be rendered
pub(crate) struct Metadata<'a> {
    pub kind: MetadataKind<'a>,
    pub dependencies: Vec<String>,
}
