use crate::model::cargo_manifest::{Package, WorkspaceConfig};

pub(crate) enum ProjectKind<'a> {
    Crate {
        package: &'a Package,
    },
    Workspace {
        config: &'a WorkspaceConfig,
        name: &'a str,
    },
}

pub(crate) struct Metadata<'a> {
    pub kind: ProjectKind<'a>,
    pub dependencies: Vec<String>,
}
