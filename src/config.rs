use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use glob::Pattern;

use crate::{
    constants::{DEFAULT_SNAPSHOT_NAME, MARKDOWN_EXTENSION, RUST_EXTENSION},
    fs::walk::is_test_file,
};

#[derive(Debug, Default)]
pub(crate) struct SnapshotOptions {
    pub format: OutputFormat,
    pub include_hidden: bool,
    pub no_tests: bool,
    pub include_cargo_toml: bool,
    pub include_workspace_toml: bool,
    pub include_readme: bool,
    pub exclude_patterns: Vec<Pattern>,
}

impl SnapshotOptions {
    /// Determines whether a file path should be excluded based on options
    pub(crate) fn should_exclude(&self, path: &Path) -> bool {
        (self.no_tests && is_test_file(path))
            || self.exclude_patterns.iter().any(|p| p.matches_path(path))
    }
}

impl TryFrom<Args> for SnapshotOptions {
    type Error = glob::PatternError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        Ok(Self {
            format: args.format,
            include_hidden: args.include_hidden,
            no_tests: args.no_tests,
            include_cargo_toml: args.include_cargo_toml,
            include_workspace_toml: args.include_workspace_toml,
            include_readme: args.include_readme,
            exclude_patterns: args
                .exclude
                .into_iter()
                .map(|s| Pattern::new(&s))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum OutputFormat {
    Rust,
    #[default]
    Markdown,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | RUST_EXTENSION => Ok(OutputFormat::Rust),
            "markdown" | MARKDOWN_EXTENSION => Ok(OutputFormat::Markdown),
            _ => Err(format!(
                "Unknown format: {s}. Use 'rust', '{RUST_EXTENSION}', 'markdown', or '{MARKDOWN_EXTENSION}'"
            )),
        }
    }
}

/// Cargo subcommand for creating a Rust code snapshot for AI analysis
#[derive(Parser, Debug)]
#[command(name = "cargo-snapshot")]
#[command(about = "Save current crate or workspace into a single file for AI analysis")]
pub(crate) struct Args {
    /// Output file (default: snapshot.rs or snapshot.md depending on format)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Output format (rust, rs, markdown, or md)
    #[arg(short, long, default_value = "markdown")]
    pub format: OutputFormat,

    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// Exclude tests from snapshot
    #[arg(long)]
    pub no_tests: bool,

    /// Exclude files matching glob pattern (can be used multiple times)
    #[arg(long, short = 'x')]
    pub exclude: Vec<String>,

    /// Include hidden files/directories
    #[arg(long)]
    pub include_hidden: bool,

    /// Snapshot only the current crate, even if it belongs to a workspace
    #[arg(long)]
    pub no_workspace: bool,

    /// Include crate Cargo.toml in snapshot
    #[arg(long)]
    pub include_cargo_toml: bool,

    /// Include workspace Cargo.toml in snapshot
    #[arg(long)]
    pub include_workspace_toml: bool,

    /// Include README.md files in snapshot
    #[arg(long)]
    pub include_readme: bool,
}

impl Args {
    /// Returns the output path with appropriate extension.
    ///
    /// If the user specified a path, it is respected. Otherwise, the default
    /// filename (snapshot) with the appropriate extension (.rs for Rust format,
    /// .md for Markdown) is used.
    ///
    /// # Examples
    /// - `--output snapshot --format rust` → `snapshot.rs`
    /// - `--output report.txt --format rust` → `report.txt` (keeps .txt)
    /// - `--output /tmp/out --format markdown` → `/tmp/out.md`
    /// - `--format markdown` (no --output) → `snapshot.md`
    pub(crate) fn output_path(&self) -> PathBuf {
        let path = match &self.output {
            Some(output) => Path::new(output).to_path_buf(),
            None => Path::new(DEFAULT_SNAPSHOT_NAME).to_path_buf(),
        };

        // If user explicitly specified an extension, respect it
        if path.extension().is_some() {
            return path;
        }

        // Otherwise add appropriate extension based on format
        match self.format {
            OutputFormat::Markdown => path.with_extension(MARKDOWN_EXTENSION),
            OutputFormat::Rust => path.with_extension(RUST_EXTENSION),
        }
    }
}
