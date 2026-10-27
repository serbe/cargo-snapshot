use tracing_subscriber::{EnvFilter, fmt};

use crate::cli::args::Args;

/// Initializes tracing using `RUST_LOG` or the configured CLI log level.
pub(crate) fn init_tracing(args: &Args) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(args.log_level.as_str()));

    fmt().with_env_filter(env_filter).init();
}
