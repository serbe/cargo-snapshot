use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

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
    pub output: Option<PathBuf>,

    /// Log level (`error`, `warn`, `info`, `debug`, or `trace`).
    #[arg(short, long, default_value = "info")]
    pub log_level: LogLevel,

    /// Include the project README in the snapshot when available.
    #[arg(long)]
    pub readme: bool,

    /// Remove comments from Rust source files in the snapshot.
    #[arg(long)]
    pub no_comments: bool,

    /// Enable Cargo features when resolving conditional modules.
    #[arg(
        short = 'F',
        long,
        value_delimiter = ',',
        conflicts_with = "all_features"
    )]
    pub features: Vec<String>,

    /// Enable all Cargo features when resolving conditional modules.
    #[arg(long, conflicts_with = "features")]
    pub all_features: bool,

    /// Do not enable default Cargo features.
    #[arg(long, conflicts_with = "all_features")]
    pub no_default_features: bool,

    /// Target triple used to evaluate target-specific cfg attributes.
    #[arg(long)]
    pub target: Option<String>,
}

impl Args {
    /// Returns the output path exactly as supplied, or the default snapshot path.
    pub(crate) fn output_path(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_SNAPSHOT_NAME))
    }

    pub(crate) fn get() -> Self {
        Self::parse_cargo_args(std::env::args_os())
    }

    fn parse_cargo_args<I>(args: I) -> Self
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut args = args.into_iter();
        let executable = args.next();
        let first_argument = args.next();
        let first_argument = first_argument.filter(|argument| argument != OsStr::new("snapshot"));

        Self::parse_from(executable.into_iter().chain(first_argument).chain(args))
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::Path};

    use super::{Args, LogLevel};

    #[test]
    fn parses_arguments_forwarded_by_cargo_subcommand() {
        let args = Args::parse_cargo_args(
            [
                "cargo-snapshot",
                "snapshot",
                "--output",
                "project.md",
                "--no-comments",
            ]
            .map(OsString::from),
        );

        assert_eq!(args.output.as_deref(), Some(Path::new("project.md")));
        assert!(args.no_comments);
        assert!(!args.readme);
        assert!(matches!(args.log_level, LogLevel::Info));
    }

    #[test]
    fn parses_arguments_when_binary_is_invoked_directly() {
        let args = Args::parse_cargo_args(
            ["cargo-snapshot", "--output", "project.md"].map(OsString::from),
        );

        assert_eq!(args.output.as_deref(), Some(Path::new("project.md")));
        assert!(!args.no_comments);
    }
}
