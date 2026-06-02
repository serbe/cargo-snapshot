use std::path::{Path, PathBuf};

use clap::Parser;

use clap::ValueEnum;
use glob::Pattern;

use crate::walk::is_test_file;

#[derive(Debug)]
pub(crate) struct SnapshotOptions {
    pub format: OutputFormat,
    pub include_hidden: bool,
    pub no_tests: bool,
    pub exclude_patterns: Vec<Pattern>,
}

impl SnapshotOptions {
    /// Determines whether a file path should be excluded based on options
    pub(crate) fn should_exclude(&self, path: &Path) -> bool {
        if self.no_tests && is_test_file(path) {
            return true;
        }

        self.exclude_patterns
            .iter()
            .any(|pattern| pattern.matches_path(path))
    }
}

impl TryFrom<Args> for SnapshotOptions {
    type Error = glob::PatternError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        Ok(Self {
            format: args.format,
            include_hidden: args.include_hidden,
            no_tests: args.no_tests,
            exclude_patterns: args
                .exclude
                .into_iter()
                .map(|s| Pattern::new(&s))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub(crate) enum OutputFormat {
    #[default]
    Rust,
    Markdown,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Ok(OutputFormat::Rust),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            _ => Err(format!("Unknown format: {s}")),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Rust => write!(f, "rust"),
            OutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// Cargo subcommand for creating a Rust code snapshot for AI analysis
#[derive(Parser, Debug)]
#[command(name = "cargo-snapshot")]
#[command(about = "Save current crate or workspace into a single file for AI analysis")]
pub(crate) struct Args {
    /// Output file (default: snapshot.rs)
    #[arg(short, long, default_value = "snapshot.rs")]
    pub output: String,

    /// Output format (rust or markdown)
    #[arg(short, long, default_value = "rust")]
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
}

impl Args {
    /// Returns the output path with the appropriate file extension based on format
    pub(crate) fn output_path(&self) -> PathBuf {
        let path = Path::new(&self.output);
        match self.format {
            OutputFormat::Markdown => path.with_extension("md"),
            OutputFormat::Rust => path.with_extension("rs"),
        }
    }
}
