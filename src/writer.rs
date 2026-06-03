use anyhow::Result;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    config::{OutputFormat, SnapshotOptions},
    manifest::Manifest,
    project::Project,
    renderer::{Renderer, markdown::MarkdownRenderer, rust::RustRenderer},
    walk::{collect_source_files, is_hidden},
};

/// Main writer for generating snapshot output
pub(crate) struct SnapshotWriter {
    options: SnapshotOptions,
    renderer: Box<dyn Renderer>,
}

impl SnapshotWriter {
    pub(crate) fn new(options: SnapshotOptions) -> Self {
        let renderer: Box<dyn Renderer> = match options.format {
            OutputFormat::Rust => Box::new(RustRenderer),
            OutputFormat::Markdown => Box::new(MarkdownRenderer),
        };
        Self { options, renderer }
    }

    /// Writes the complete snapshot to the output path
    pub(crate) fn write(&self, project: &Project, output: &Path) -> Result<()> {
        let mut file = File::create(output)?;

        self.renderer.write_header(&mut file)?;
        self.write_metadata(&mut file, &project.manifest)?;
        self.write_sources(&mut file, project)?;

        Ok(())
    }

    /// Writes metadata section with package/workspace information using the renderer
    fn write_metadata<W: Write>(&self, output: &mut W, manifest: &Manifest) -> Result<()> {
        self.renderer.write_metadata(output)?;

        if let Some(package) = &manifest.cargo_toml.package {
            self.renderer.write_package_metadata(output, package)?;
        } else if manifest.is_workspace() {
            self.renderer.write_workspace_metadata(output, manifest)?;
        }

        let dependencies = manifest.dependencies();

        if !dependencies.is_empty() {
            self.renderer.write_dependencies(output, &dependencies)?;
        }

        writeln!(output)?;
        Ok(())
    }

    /// Writes all source files based on project type
    fn write_sources<W: Write>(&self, output: &mut W, project: &Project) -> Result<()> {
        if project.is_workspace_root() {
            self.write_workspace_sources(output, project)?;
        } else {
            self.write_single_crate_structure(output, project)?;
            self.write_crate_sources(
                output,
                &project.manifest.crate_name(),
                project.root_dir.join("src"),
                &project.root_dir,
            )?;
        }
        Ok(())
    }

    /// Writes directory structure for a single crate
    fn write_single_crate_structure(
        &self,
        output: &mut impl Write,
        project: &Project,
    ) -> Result<()> {
        match self.options.format {
            OutputFormat::Rust => {
                writeln!(output, "// ========== PROJECT STRUCTURE ==========\n")?;
                writeln!(output, "// {}/", project.manifest.crate_name())?;
                writeln!(output, "// └── src/")?;
                self.print_directory_structure(output, &project.root_dir.join("src"), "//     ")?;
            }
            OutputFormat::Markdown => {
                writeln!(output, "## Project Structure\n")?;
                writeln!(output, "```")?;
                writeln!(output, "{}/", project.manifest.crate_name())?;
                writeln!(output, "└── src/")?;
                self.print_directory_structure_markdown(
                    output,
                    &project.root_dir.join("src"),
                    "    ",
                )?;
                writeln!(output, "```")?;
            }
        }
        writeln!(output)?;
        Ok(())
    }

    /// Prints directory structure for Markdown output (without comment prefixes)
    fn print_directory_structure_markdown(
        &self,
        output: &mut impl Write,
        dir: &Path,
        prefix: &str,
    ) -> Result<()> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(Result::ok)
            .filter(|e| {
                let path = e.path();
                let is_rs = path.extension().is_some_and(|ext| ext == "rs");
                let is_dir = path.is_dir();

                if !self.options.include_hidden && is_hidden(&path) {
                    return false;
                }

                is_rs || is_dir
            })
            .collect();

        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for (i, entry) in entries.iter().enumerate() {
            let is_last = i == entries.len() - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if entry.path().is_dir() {
                writeln!(output, "{prefix}{connector}{name_str}/")?;
                let new_prefix = if is_last {
                    format!("{prefix}    ")
                } else {
                    format!("{prefix}│   ")
                };
                self.print_directory_structure_markdown(output, &entry.path(), &new_prefix)?;
            } else {
                writeln!(output, "{prefix}{connector}{name_str}")?;
            }
        }
        Ok(())
    }

    /// Prints directory structure for Rust output (with comment prefixes)
    fn print_directory_structure(
        &self,
        output: &mut impl Write,
        dir: &Path,
        prefix: &str,
    ) -> Result<()> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(Result::ok)
            .filter(|e| {
                let path = e.path();
                let is_rs = path.extension().is_some_and(|ext| ext == "rs");
                let is_dir = path.is_dir();

                if !self.options.include_hidden && is_hidden(&path) {
                    return false;
                }

                is_rs || is_dir
            })
            .collect();

        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for (i, entry) in entries.iter().enumerate() {
            let is_last = i == entries.len() - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if entry.path().is_dir() {
                writeln!(output, "{prefix}{connector}{name_str}/")?;
                let new_prefix = if is_last {
                    format!("{prefix}    ")
                } else {
                    format!("{prefix}│   ")
                };
                self.print_directory_structure(output, &entry.path(), &new_prefix)?;
            } else {
                writeln!(output, "{prefix}{connector}{name_str}")?;
            }
        }
        Ok(())
    }

    /// Writes all sources for a workspace project
    fn write_workspace_sources(&self, output: &mut impl Write, project: &Project) -> Result<()> {
        match self.options.format {
            OutputFormat::Rust => {
                writeln!(
                    output,
                    "// ========== WORKSPACE: {} ==========\n",
                    project.workspace_name(),
                )?;
            }
            OutputFormat::Markdown => {
                writeln!(output, "# Workspace: {}\n", project.workspace_name(),)?;
            }
        }

        self.write_project_structure(output, project)?;

        for member in &project.members {
            self.write_crate_sources(output, &member.name, member.src_dir(), &project.root_dir)?;
        }

        Ok(())
    }

    /// Writes the project structure tree
    fn write_project_structure(&self, output: &mut impl Write, project: &Project) -> Result<()> {
        match self.options.format {
            OutputFormat::Rust => {
                writeln!(output, "// ========== PROJECT STRUCTURE ==========\n")?;
                let root_name = project.root_dir.file_name().unwrap_or_default();
                writeln!(output, "// {}", root_name.to_string_lossy())?;
                for member in &project.members {
                    let relative = member
                        .absolute_path
                        .strip_prefix(&project.root_dir)
                        .unwrap_or(&member.absolute_path);
                    writeln!(output, "// ├── {}/", relative.display())?;
                    self.print_directory_structure(output, &member.src_dir(), "// │   ")?;
                }
            }
            OutputFormat::Markdown => {
                writeln!(output, "## Project Structure\n")?;
                writeln!(output, "```")?;
                let root_name = project.root_dir.file_name().unwrap_or_default();
                writeln!(output, "{}", root_name.to_string_lossy())?;
                for member in &project.members {
                    let relative = member
                        .absolute_path
                        .strip_prefix(&project.root_dir)
                        .unwrap_or(&member.absolute_path);
                    writeln!(output, "├── {}/", relative.display())?;
                    self.print_directory_structure_markdown(output, &member.src_dir(), "│   ")?;
                }
                writeln!(output, "```\n")?;
            }
        }
        writeln!(output)?;
        Ok(())
    }

    /// Writes all source files for a single crate
    fn write_crate_sources(
        &self,
        output: &mut impl Write,
        crate_name: &str,
        src_dir: PathBuf,
        root_dir: &Path,
    ) -> Result<()> {
        match self.options.format {
            OutputFormat::Rust => {
                writeln!(output, "// ========== CRATE: {crate_name} ==========\n")?;
            }
            OutputFormat::Markdown => {
                writeln!(output, "## Crate: {crate_name}\n")?;
            }
        }

        let files = collect_source_files(&src_dir, &self.options)?;

        for path in files {
            self.write_file(output, root_dir, &path)?;
        }
        Ok(())
    }

    /// Writes a single file with its content to the output
    fn write_file(&self, output: &mut impl Write, root_dir: &Path, file_path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let relative = file_path.strip_prefix(root_dir).unwrap_or(file_path);
        let normalized_path = relative.to_str().unwrap_or("").replace('\\', "/");
        let line_count = content.lines().count();

        self.renderer
            .begin_file(output, Path::new(&normalized_path), line_count)?;
        output.write_all(content.as_bytes())?;
        self.renderer.end_file(output)?;
        Ok(())
    }
}
