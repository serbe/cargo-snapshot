use std::path::{Path, PathBuf};

use glob::Pattern;

use crate::{
    SnapshotResult,
    cli::{args::Args, format::OutputFormat},
    fs::filters::is_test_file,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct Config {
    pub format: OutputFormat,
    pub include_hidden: bool,
    pub no_tests: bool,
    pub no_workspace: bool,
    pub include_cargo_toml: bool,
    pub include_workspace_toml: bool,
    pub include_readme: bool,
    pub exclude_patterns: Vec<Pattern>,
    pub output_path: PathBuf,
}

impl Config {
    pub(crate) fn new(args: Args) -> SnapshotResult<Self> {
        Ok(Self {
            format: args.format,
            include_hidden: args.include_hidden,
            no_tests: args.no_tests,
            no_workspace: args.no_workspace,
            include_cargo_toml: args.include_cargo_toml,
            include_workspace_toml: args.include_workspace_toml,
            include_readme: args.include_readme,
            output_path: args.output_path(),
            exclude_patterns: args
                .exclude
                .into_iter()
                .map(|s| Pattern::new(&s))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    /// Determines whether a file path should be excluded based on options
    pub(crate) fn should_exclude(&self, path: &Path) -> bool {
        (self.no_tests && is_test_file(path))
            || self.exclude_patterns.iter().any(|p| p.matches_path(path))
    }
}
