use std::path::{Path, PathBuf};

use clap::Parser;

use crate::{
    cli::format::OutputFormat,
    config::{DEFAULT_SNAPSHOT_NAME, MARKDOWN_EXTENSION, RUST_EXTENSION},
};

#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub(crate) enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
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
    pub log_level: LogLevel,

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

    pub(crate) fn get() -> Self {
        Args::parse()
    }
}
