use crate::model::cargo_manifest::{Package, WorkspaceManifest};

pub(crate) enum ProjectKind<'a> {
    Crate {
        package: &'a Package,
    },
    Workspace {
        config: &'a WorkspaceManifest,
        name: &'a str,
    },
}

pub(crate) struct Metadata<'a> {
    pub kind: ProjectKind<'a>,
    pub dependencies: Vec<String>,
}
