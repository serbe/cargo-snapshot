use anyhow::Result;
use std::{io::Write, path::Path};

use crate::manifest::Manifest;

pub(crate) mod markdown;
pub(crate) mod rust;

pub(crate) struct FileHeader<'a> {
    pub path: &'a Path,
    pub lines: usize,
}

/// Trait for rendering file content in different output formats
pub(crate) trait Renderer {
    fn begin_file(&self, out: &mut dyn Write, path: &Path, lines: usize) -> Result<()>;
    fn end_file(&self, out: &mut dyn Write) -> Result<()>;

    /// Renders the header section
    fn write_header(&self, out: &mut dyn Write) -> Result<()> {
        Ok(())
    }

    fn write_metadata(&self, out: &mut dyn Write) -> Result<()> {
        Ok(())
    }

    /// Renders metadata section for a package
    fn write_package_metadata(
        &self,
        out: &mut dyn Write,
        package: &crate::cargo_toml::Package,
    ) -> Result<()> {
        Ok(())
    }

    /// Renders metadata section for a workspace
    fn write_workspace_metadata(&self, out: &mut dyn Write, manifest: &Manifest) -> Result<()> {
        Ok(())
    }

    /// Renders dependencies list
    fn write_dependencies(&self, out: &mut dyn Write, dependencies: &[String]) -> Result<()> {
        Ok(())
    }
}
