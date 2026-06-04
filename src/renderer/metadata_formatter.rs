use anyhow::Result;
use std::io::Write;

use super::Metadata;
use crate::{
    cargo_toml::{Package, WorkspaceConfig},
    metadata::MetadataKind,
};

pub(crate) struct MetadataFormatter<'a, W: Write + ?Sized> {
    out: &'a mut W,
    prefix: &'a str,
}

impl<'a, W: ?Sized + Write> MetadataFormatter<'a, W> {
    pub(crate) fn new(out: &'a mut W, prefix: &'a str) -> Self {
        Self { out, prefix }
    }

    fn write_field(&mut self, key: &str, value: Option<&str>) -> Result<()> {
        if let Some(v) = value {
            writeln!(self.out, "{}{key}: {v}", self.prefix)?;
        }
        Ok(())
    }

    pub(crate) fn format(&mut self, metadata: &Metadata<'_>) -> Result<()> {
        match &metadata.kind {
            MetadataKind::Crate { package } => {
                self.format_package(package)?;
            }
            MetadataKind::Workspace { config, name } => {
                self.format_workspace(config, Some(name))?;
            }
        }

        if !metadata.dependencies.is_empty() {
            self.format_dependencies(&metadata.dependencies)?;
        }

        writeln!(self.out)?;
        Ok(())
    }

    fn format_package(&mut self, pkg: &Package) -> Result<()> {
        writeln!(self.out, "{}name: {}", self.prefix, pkg.name)?;
        self.write_field("version", pkg.version.as_deref())?;
        self.write_field("edition", pkg.edition.as_deref())?;
        self.write_field("license", pkg.license.as_deref())?;
        self.write_field("description", pkg.description.as_deref())?;
        Ok(())
    }

    fn format_workspace(&mut self, ws: &WorkspaceConfig, name: Option<&str>) -> Result<()> {
        if let Some(name) = name {
            self.write_field("workspace", Some(name))?;
        }
        writeln!(self.out, "{}type: workspace", self.prefix)?;

        if !ws.members.is_empty() {
            writeln!(
                self.out,
                "{}members: {}",
                self.prefix,
                ws.members.join(", ")
            )?;
        }
        self.write_field("resolver", ws.resolver.as_deref())?;

        if let Some(pkg) = &ws.package {
            self.write_field("version", pkg.version.as_deref())?;
            self.write_field("edition", pkg.edition.as_deref())?;
            self.write_field("license", pkg.license.as_deref())?;
        }
        Ok(())
    }

    fn format_dependencies(&mut self, deps: &[String]) -> Result<()> {
        let deps_str = deps.join(", ");
        writeln!(self.out, "{}dependencies: {deps_str}", self.prefix)?;
        Ok(())
    }
}
