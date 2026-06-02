use std::{fmt::Display, io::Write, path::Path, str::FromStr};

use anyhow::Result;

use crate::{
    cargo_toml::Package,
    config::SnapshotOptions,
    manifest::Manifest,
    renderer::{markdown::MarkdownRenderer, rust::RustRenderer},
    workspace::WorkspaceMember,
};

mod markdown;
mod rust;
mod tree;

use tree::print_directory_tree;

/// Тип-тег для определения формата вывода
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Rust,
    Markdown,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Rust
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Ok(OutputFormat::Rust),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            _ => Err(format!("Unknown format: {s}")),
        }
    }
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Rust => write!(f, "rust"),
            OutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Внутренний трейт для реализации рендереров
pub(crate) trait RendererImpl {
    fn line_prefix(&self) -> &str;
    fn begin_file<W: Write>(&self, out: &mut W, path: &Path, lines: usize) -> Result<()>;
    fn end_file<W: Write>(&self, out: &mut W) -> Result<()>;
    fn write_header<W: Write>(&self, out: &mut W) -> Result<()>;
    fn write_package_metadata<W: Write>(&self, out: &mut W, package: &Package) -> Result<()>;
    fn write_workspace_metadata<W: Write>(&self, out: &mut W, manifest: &Manifest) -> Result<()>;
    fn write_dependencies<W: Write>(&self, out: &mut W, dependencies: &[String]) -> Result<()>;

    fn write_crate_structure<W: Write>(
        &self,
        out: &mut W,
        crate_name: &str,
        src_path: &Path,
        options: &SnapshotOptions,
    ) -> Result<()> {
        let prefix = self.line_prefix();
        writeln!(out, "{}{}/", prefix, crate_name)?;
        writeln!(out, "{}└── src/", prefix)?;

        print_directory_tree(out, src_path, "    ", prefix, options)?;
        Ok(())
    }

    fn write_workspace_structure<W: Write>(
        &self,
        out: &mut W,
        root_name: &str,
        members: &[WorkspaceMember],
        root_dir: &Path,
        options: &SnapshotOptions,
    ) -> Result<()> {
        let prefix = self.line_prefix();
        writeln!(out, "{}{}", prefix, root_name)?;

        for member in members {
            let relative = member
                .absolute_path
                .strip_prefix(root_dir)
                .unwrap_or(&member.absolute_path);
            writeln!(out, "{prefix}├── {}/", relative.display())?;
            print_directory_tree(out, &member.src_dir(), "│   ", prefix, options)?;
        }
        Ok(())
    }
}

/// Публичный enum рендерера
pub enum Renderer {
    Rust(RustRenderer),
    Markdown(MarkdownRenderer),
}

impl Renderer {
    pub fn line_prefix(&self) -> &str {
        match self {
            Renderer::Rust(r) => r.line_prefix(),
            Renderer::Markdown(r) => r.line_prefix(),
        }
    }

    pub fn begin_file<W: Write>(&self, out: &mut W, path: &Path, lines: usize) -> Result<()> {
        match self {
            Renderer::Rust(r) => r.begin_file(out, path, lines),
            Renderer::Markdown(r) => r.begin_file(out, path, lines),
        }
    }

    pub fn end_file<W: Write>(&self, out: &mut W) -> Result<()> {
        match self {
            Renderer::Rust(r) => r.end_file(out),
            Renderer::Markdown(r) => r.end_file(out),
        }
    }

    pub fn write_header<W: Write>(&self, out: &mut W) -> Result<()> {
        match self {
            Renderer::Rust(r) => r.write_header(out),
            Renderer::Markdown(r) => r.write_header(out),
        }
    }

    pub fn write_package_metadata<W: Write>(&self, out: &mut W, package: &Package) -> Result<()> {
        match self {
            Renderer::Rust(r) => r.write_package_metadata(out, package),
            Renderer::Markdown(r) => r.write_package_metadata(out, package),
        }
    }

    pub fn write_workspace_metadata<W: Write>(
        &self,
        out: &mut W,
        manifest: &Manifest,
    ) -> Result<()> {
        match self {
            Renderer::Rust(r) => r.write_workspace_metadata(out, manifest),
            Renderer::Markdown(r) => r.write_workspace_metadata(out, manifest),
        }
    }

    pub fn write_dependencies<W: Write>(&self, out: &mut W, dependencies: &[String]) -> Result<()> {
        match self {
            Renderer::Rust(r) => r.write_dependencies(out, dependencies),
            Renderer::Markdown(r) => r.write_dependencies(out, dependencies),
        }
    }

    pub fn write_crate_structure<W: Write>(
        &self,
        out: &mut W,
        crate_name: &str,
        src_path: &Path,
        options: &SnapshotOptions,
    ) -> Result<()> {
        match self {
            Renderer::Rust(r) => r.write_crate_structure(out, crate_name, src_path, options),
            Renderer::Markdown(r) => r.write_crate_structure(out, crate_name, src_path, options),
        }
    }

    pub fn write_workspace_structure<W: Write>(
        &self,
        out: &mut W,
        root_name: &str,
        members: &[WorkspaceMember],
        root_dir: &Path,
        options: &SnapshotOptions,
    ) -> Result<()> {
        match self {
            Renderer::Rust(r) => {
                r.write_workspace_structure(out, root_name, members, root_dir, options)
            }
            Renderer::Markdown(r) => {
                r.write_workspace_structure(out, root_name, members, root_dir, options)
            }
        }
    }
}

/// Фабрика для создания рендерера
pub fn create_renderer(format: OutputFormat) -> Renderer {
    match format {
        OutputFormat::Rust => Renderer::Rust(RustRenderer),
        OutputFormat::Markdown => Renderer::Markdown(MarkdownRenderer),
    }
}
