pub(crate) use crate::error::SnapshotResult;
use crate::{
    cli::args::Args,
    config::{settings::Config, tracing::init_tracing},
    error::SnapshotError,
    generator::snapshot_generator::SnapshotGenerator,
    snapshot::Snapshot,
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
mod snapshot;

fn main() -> Result<(), SnapshotError> {
    let args = Args::get();

    init_tracing(&args);

    let config = Config::new(args)?;
    let output = config.output_path.clone().display().to_string();

    // SnapshotGenerator::new(config).write()?;

    let snapshot = Snapshot::new()?;

    dbg!(&snapshot.metadata);

    // println!("✅ Snapshot saved to {output}");
    Ok(())
}
