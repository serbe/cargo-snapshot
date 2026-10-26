use std::path::{Path, PathBuf};

use clap::Parser;

pub(crate) const DEFAULT_SNAPSHOT_NAME: &str = "snapshot.md";

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

/// Cargo subcommand that writes a project snapshot for AI analysis.
#[derive(Parser, Debug)]
#[command(name = "cargo-snapshot")]
#[command(about = "Create a Markdown snapshot of the current Cargo project for AI analysis")]
pub(crate) struct Args {
    /// Output file (default: snapshot.md).
    #[arg(short, long)]
    pub output: Option<String>,

    /// Log level (`error`, `warn`, `info`, `debug`, or `trace`).
    #[arg(short, long, default_value = "info")]
    pub log_level: LogLevel,

    /// Include the project README in the snapshot when available.
    #[arg(long)]
    pub readme: bool,

    /// Remove comments from Rust source files in the snapshot.
    #[arg(long)]
    pub no_comments: bool,
}

impl Args {
    /// Returns the output path, adding an extension when the path has none.
    ///
    /// An explicitly supplied extension is preserved. Otherwise, the extension
    /// is selected by `--format`; the generated snapshot content is Markdown.
    pub(crate) fn output_path(&self) -> PathBuf {
        match &self.output {
            Some(output) => Path::new(output).to_path_buf(),
            None => Path::new(DEFAULT_SNAPSHOT_NAME).to_path_buf(),
        }
    }

    pub(crate) fn get() -> Self {
        Args::parse()
    }
}
