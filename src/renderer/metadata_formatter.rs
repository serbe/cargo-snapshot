use std::io::Write;

use crate::{
    SnapshotResult,
    model::{
        cargo_manifest::{Package, WorkspaceConfig},
        metadata::{Metadata, ProjectKind},
    },
};

pub(crate) struct MetadataFormatter<'a, W: Write + ?Sized> {
    out: &'a mut W,
    prefix: &'a str,
}

impl<'a, W: ?Sized + Write> MetadataFormatter<'a, W> {
    pub(crate) fn new(out: &'a mut W, prefix: &'a str) -> Self {
        Self { out, prefix }
    }

    fn write_field(&mut self, key: &str, value: &str) -> SnapshotResult<()> {
        writeln!(self.out, "{}{key}: {value}", self.prefix)?;
        Ok(())
    }

    fn write_opt_field(&mut self, key: &str, value: Option<&str>) -> SnapshotResult<()> {
        if let Some(v) = value {
            self.write_field(key, v)?;
        }
        Ok(())
    }

    fn write_vec_field(&mut self, key: &str, value: &[String]) -> SnapshotResult<()> {
        if !value.is_empty() {
            self.write_field(key, &value.join(", "))?;
        }
        Ok(())
    }

    pub(crate) fn format(&mut self, metadata: &Metadata<'_>) -> SnapshotResult<()> {
        match &metadata.kind {
            ProjectKind::Crate { package } => {
                self.format_package(package)?;
            }
            ProjectKind::Workspace { config, name } => {
                self.format_workspace(config, name)?;
            }
        }

        if !metadata.dependencies.is_empty() {
            self.write_vec_field("dependencies", &metadata.dependencies)?;
        }

        writeln!(self.out)?;
        Ok(())
    }

    fn format_package(&mut self, pkg: &Package) -> SnapshotResult<()> {
        self.write_opt_field("name", pkg.name.as_deref())?;
        self.write_opt_field("version", pkg.version.as_deref())?;
        self.write_opt_field("edition", pkg.edition.as_deref())?;
        self.write_opt_field("license", pkg.license.as_deref())?;
        self.write_opt_field("description", pkg.description.as_deref())?;
        Ok(())
    }

    fn format_workspace(&mut self, ws: &WorkspaceConfig, name: &str) -> SnapshotResult<()> {
        self.write_field("workspace", name)?;
        self.write_vec_field("members", &ws.members)?;
        self.write_opt_field("resolver", ws.resolver.as_deref())?;

        if let Some(pkg) = &ws.package {
            self.write_opt_field("version", pkg.version.as_deref())?;
            self.write_opt_field("edition", pkg.edition.as_deref())?;
            self.write_opt_field("license", pkg.license.as_deref())?;
        }
        Ok(())
    }
}
