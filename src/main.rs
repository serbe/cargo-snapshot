use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

use config::Args;

pub(crate) use crate::error::SnapshotResult;
use crate::{model::project::Project, writer::SnapshotWriter};

mod config;
mod constants;
mod error;
mod fs;
mod model;
mod renderer;
mod writer;

fn main() -> SnapshotResult<()> {
    let args = Args::parse();

    init_tracing(&args);

    let output_path = args.output_path();
    let no_workspace = args.no_workspace;
    let options = args.try_into()?;

    let project = Project::from_current_dir(no_workspace)?;

    SnapshotWriter::new(options).write(&project, &output_path)?;

    println!("✅ Snapshot saved to {}", output_path.display());
    Ok(())
}

/// Initialize tracing subscriber with configured log level
fn init_tracing(args: &Args) {
    let env_filter =
        EnvFilter::try_from_default_env().map_or(EnvFilter::new(&args.log_level), |f| f);

    fmt().with_env_filter(env_filter).init();
}
