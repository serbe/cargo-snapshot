use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

use config::Args;
use project::Project;

use crate::writer::SnapshotWriter;

mod cargo_toml;
mod config;
mod manifest;
mod project;
// mod renderer;
mod walk;
mod workspace;
mod writer;

fn main() -> Result<()> {
    let args = Args::parse();

    init_tracing(&args)?;

    let output_path = args.output_path();
    let options = args.try_into()?;

    let project = Project::from_current_dir()?;

    SnapshotWriter::new(options).write(&project, &output_path)?;

    println!("✅ Snapshot saved to {}", output_path.display());
    Ok(())
}

/// Initialize tracing subscriber with configured log level
fn init_tracing(args: &Args) -> Result<()> {
    let env_filter =
        EnvFilter::try_from_default_env().map_or(EnvFilter::new(&args.log_level), |f| f);

    fmt().with_env_filter(env_filter).init();
    Ok(())
}
