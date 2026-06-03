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
    /// Returns the output path with appropriate extension.
    ///
    /// If the user specified a path with an explicit extension (e.g., `report.txt`),
    /// it is respected and no extension is changed. Otherwise, the appropriate
    /// extension (`.rs` for Rust format, `.md` for Markdown) is added.
    ///
    /// # Examples
    /// - `--output snapshot --format rust` → `snapshot.rs`
    /// - `--output report.txt --format rust` → `report.txt` (keeps .txt)
    /// - `--output /tmp/out --format markdown` → `/tmp/out.md`
    pub(crate) fn output_path(&self) -> PathBuf {
        let path = Path::new(&self.output);

        // If user explicitly specified an extension, respect it
        if path.extension().is_some() {
            return path.to_path_buf();
        }

        // Otherwise add appropriate extension based on format
        match self.format {
            OutputFormat::Markdown => path.with_extension("md"),
            OutputFormat::Rust => path.with_extension("rs"),
        }
    }
}
