use std::io::Write;

use crate::{
    SnapshotResult,
    config::OutputFormat,
    model::metadata::Metadata,
    renderer::{markdown::MarkdownRenderer, rust::RustRenderer},
};

pub(crate) mod markdown;
pub(crate) mod metadata_formatter;
pub(crate) mod rust;

/// Trait for rendering snapshot output in different formats
pub(crate) trait Renderer: Send + Sync {
    /// Renders the main header (e.g., "CARGO SNAPSHOT" banner)
    fn render_header(&self, out: &mut dyn Write) -> SnapshotResult<()>;

    /// Renders metadata section from structured data
    fn render_metadata(&self, out: &mut dyn Write, metadata: &Metadata<'_>) -> SnapshotResult<()>;

    /// Renders a single file with its content
    fn render_file(&self, out: &mut dyn Write, path: &str, content: &str) -> SnapshotResult<()>;

    /// Renders a crate heading
    fn render_crate_heading(&self, out: &mut dyn Write, name: &str) -> SnapshotResult<()>;

    /// Renders a workspace heading
    fn render_workspace_heading(&self, out: &mut dyn Write, name: &str) -> SnapshotResult<()>;

    /// Renders the beginning of project structure tree
    fn render_structure_begin(&self, out: &mut dyn Write) -> SnapshotResult<()>;

    /// Renders the end of project structure tree
    fn render_structure_end(&self, out: &mut dyn Write) -> SnapshotResult<()>;

    /// Renders a root directory in the structure tree
    fn render_structure_root(&self, out: &mut dyn Write, name: &str) -> SnapshotResult<()>;

    /// Renders a member directory in the structure tree
    fn render_structure_member(&self, out: &mut dyn Write, path: &str) -> SnapshotResult<()>;

    /// Returns the prefix string for tree lines
    fn tree_prefix(&self) -> &'static str;

    fn metadata_prefix(&self) -> &'static str;
}

/// Factory function to create renderer by format
pub(crate) fn create_renderer(format: OutputFormat) -> Box<dyn Renderer> {
    match format {
        OutputFormat::Rust => Box::new(RustRenderer),
        OutputFormat::Markdown => Box::new(MarkdownRenderer),
    }
}
