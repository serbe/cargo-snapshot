use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

use args::Args;
use project::Project;

mod args;
mod manifest;
mod project;
mod structure;
mod walk;
mod workspace;

fn main() -> Result<()> {
    let args = Args::parse();

    init_tracing(&args)?;

    let project = Project::from_current_dir(args)?;
    let output_path = project.args.output_path();

    project.collect_sources(&output_path)?;

    println!("✅ Snapshot saved to {}", output_path.display());
    Ok(())
}

/// Initialize tracing subscriber with configured log level
fn init_tracing(args: &Args) -> Result<()> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    fmt().with_env_filter(env_filter).init();
    Ok(())
}
