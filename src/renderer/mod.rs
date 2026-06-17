use std::io::Write;

use crate::{
    SnapshotResult,
    config::OutputFormat,
    metadata::Metadata,
    renderer::{markdown::MarkdownRenderer, rust::RustRenderer},
};

pub(crate) mod markdown;
pub(crate) mod metadata_formatter;
pub(crate) mod rust;

pub(crate) struct RenderStyle {
    /// Prefix for every metadata/tree line (e.g. "// " for Rust, "" for Markdown)
    pub line_prefix: &'static str,
    /// Prefix used by `MetadataFormatter` for key-value pairs
    pub metadata_key_prefix: &'static str,
    /// Opening fence/wrapper before a file's content (path is interpolated by caller)
    pub file_open: fn(&str, &str) -> String,
    /// Closing fence/wrapper after a file's content
    pub file_close: &'static str,
    pub crate_heading: fn(&str) -> String,
    pub workspace_heading: fn(&str) -> String,
    pub structure_begin: &'static str,
    pub structure_end: &'static str,
    pub structure_root: fn(&str) -> String,
    pub structure_member: fn(&str) -> String,
}

/// Trait for rendering snapshot output in different formats
pub(crate) trait Renderer: Send + Sync {
    /// Returns the style parameters for this format
    fn style(&self) -> RenderStyle;

    /// Renders the main header (e.g., "CARGO SNAPSHOT" banner)
    fn render_header(&self, out: &mut dyn Write) -> SnapshotResult<()>;

    /// Renders metadata section from structured data
    fn render_metadata(&self, out: &mut dyn Write, metadata: &Metadata<'_>) -> SnapshotResult<()> {
        let style = self.style();
        let mut formatter =
            metadata_formatter::MetadataFormatter::new(out, style.metadata_key_prefix);
        formatter.format(metadata)
    }

    /// Renders a single file with its content
    fn render_file(&self, out: &mut dyn Write, path: &str, content: &str) -> SnapshotResult<()> {
        let style = self.style();
        writeln!(out, "{}", (style.file_open)(path, content))?;
        out.write_all(content.as_bytes())?;
        writeln!(out, "{}", style.file_close)?;
        Ok(())
    }

    /// Renders a crate heading
    fn render_crate_heading(&self, out: &mut dyn Write, name: &str) -> SnapshotResult<()> {
        writeln!(out, "{}", (self.style().crate_heading)(name))?;
        Ok(())
    }

    /// Renders a workspace heading
    fn render_workspace_heading(&self, out: &mut dyn Write, name: &str) -> SnapshotResult<()> {
        writeln!(out, "{}", (self.style().workspace_heading)(name))?;
        Ok(())
    }

    /// Renders the beginning of project structure tree
    fn render_structure_begin(&self, out: &mut dyn Write) -> SnapshotResult<()> {
        writeln!(out, "{}", self.style().structure_begin)?;
        Ok(())
    }

    /// Renders the end of project structure tree
    fn render_structure_end(&self, out: &mut dyn Write) -> SnapshotResult<()> {
        writeln!(out, "{}", self.style().structure_end)?;
        Ok(())
    }

    /// Renders a root directory in the structure tree
    fn render_structure_root(&self, out: &mut dyn Write, name: &str) -> SnapshotResult<()> {
        writeln!(out, "{}", (self.style().structure_root)(name))?;
        Ok(())
    }

    /// Renders a member directory in the structure tree
    fn render_structure_member(&self, out: &mut dyn Write, path: &str) -> SnapshotResult<()> {
        writeln!(out, "{}", (self.style().structure_member)(path))?;
        Ok(())
    }

    /// Returns the prefix string for tree lines
    fn tree_prefix(&self) -> &'static str {
        self.style().line_prefix
    }
}

/// Factory function to create renderer by format
pub(crate) fn create_renderer(format: OutputFormat) -> Box<dyn Renderer> {
    match format {
        OutputFormat::Rust => Box::new(RustRenderer),
        OutputFormat::Markdown => Box::new(MarkdownRenderer),
    }
}
