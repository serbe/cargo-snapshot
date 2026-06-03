use anyhow::Result;
use std::io::Write;

use crate::{
    cargo_toml::{Package, WorkspaceConfig},
    config::OutputFormat,
    renderer::{markdown::MarkdownRenderer, rust::RustRenderer},
};

pub(crate) mod markdown;
pub(crate) mod rust;

/// Metadata structure containing all information to be rendered
pub(crate) struct Metadata<'a> {
    pub package: Option<&'a Package>,
    pub workspace: Option<&'a WorkspaceConfig>,
    pub workspace_name: Option<String>,
    pub dependencies: Vec<String>,
}

/// Trait for rendering snapshot output in different formats
pub(crate) trait Renderer: Send + Sync {
    /// Renders the main header (e.g., "CARGO SNAPSHOT" banner)
    fn render_header(&self, out: &mut dyn Write) -> Result<()>;

    /// Renders metadata section from structured data
    fn render_metadata(&self, out: &mut dyn Write, metadata: &Metadata<'_>) -> Result<()>;

    /// Renders a single file with its content
    fn render_file(&self, out: &mut dyn Write, path: &str, content: &str) -> Result<()>;

    /// Renders a crate heading
    fn render_crate_heading(&self, out: &mut dyn Write, name: &str) -> Result<()>;

    /// Renders a workspace heading
    fn render_workspace_heading(&self, out: &mut dyn Write, name: &str) -> Result<()>;

    /// Renders the beginning of project structure tree
    fn render_structure_begin(&self, out: &mut dyn Write) -> Result<()>;

    /// Renders the end of project structure tree
    fn render_structure_end(&self, out: &mut dyn Write) -> Result<()>;

    /// Renders a root directory in the structure tree
    fn render_structure_root(&self, out: &mut dyn Write, name: &str) -> Result<()>;

    /// Renders a member directory in the structure tree
    fn render_structure_member(&self, out: &mut dyn Write, path: &str) -> Result<()>;

    /// Returns the prefix string for tree lines (e.g., "// " for Rust, "" for Markdown)
    fn tree_prefix(&self) -> &'static str;
}

/// Factory function to create renderer by format
pub(crate) fn create_renderer(format: OutputFormat) -> Box<dyn Renderer> {
    match format {
        OutputFormat::Rust => Box::new(RustRenderer::new()),
        OutputFormat::Markdown => Box::new(MarkdownRenderer::new()),
    }
}
