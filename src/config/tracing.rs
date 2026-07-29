use tracing_subscriber::{EnvFilter, fmt};

use crate::cli::args::Args;

/// Initialize tracing subscriber with configured log level
pub(crate) fn init_tracing(args: &Args) {
    let env_filter =
        EnvFilter::try_from_default_env().map_or(EnvFilter::new(&args.log_level), |f| f);

    fmt().with_env_filter(env_filter).init();
}
