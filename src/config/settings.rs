use std::path::PathBuf;

use crate::cli::args::Args;

#[derive(Clone, Debug, Default)]
pub(crate) struct Config {
    pub output_path: PathBuf,
    pub readme: bool,
    pub no_comments: bool,
}

impl Config {
    pub(crate) fn new(args: Args) -> Self {
        Self {
            output_path: args.output_path(),
            readme: args.readme,
            no_comments: args.no_comments,
        }
    }
}
