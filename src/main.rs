pub(crate) use crate::error::SnapshotResult;
use crate::{
    cli::args::Args,
    config::{settings::Config, tracing::init_tracing},
    generator::snapshot_generator::SnapshotGenerator,
};

mod cli;
mod config;
mod core;
mod discovery;
mod error;
mod formatter;
mod fs;
mod generator;
mod model;
mod renderer;

fn main() -> SnapshotResult<()> {
    let args = Args::get();

    init_tracing(&args);

    let config = Config::new(args)?;
    let output = config.output_path.clone().display().to_string();

    SnapshotGenerator::new(config).write()?;

    println!("✅ Snapshot saved to {output}");
    Ok(())
}
