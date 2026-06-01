use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
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
            _ => Err(format!("Unknown format: {}", s)),
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
pub struct Args {
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
    /// Check if a path should be excluded based on filters
    pub fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        if self.no_tests && is_test_file(path) {
            return true;
        }

        self.exclude.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .ok()
                .is_some_and(|glob| glob.matches(&path_str))
        })
    }

    /// Get output path with correct extension based on format
    pub fn output_path(&self) -> PathBuf {
        let path = Path::new(&self.output);
        match self.format {
            OutputFormat::Markdown => path.with_extension("md"),
            OutputFormat::Rust => path.with_extension("rs"),
        }
    }
}

/// Check if a file path corresponds to a test file
fn is_test_file(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "tests")
        || path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "test" || s.ends_with("_test"))
}
