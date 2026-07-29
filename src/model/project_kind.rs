use crate::core::package::{Package, WorkspaceConfig};

pub(crate) enum ProjectKind<'a> {
    Crate {
        package: &'a Package,
    },
    Workspace {
        config: &'a WorkspaceConfig,
        name: &'a str,
    },
}
