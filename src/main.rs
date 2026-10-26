use crate::{error::SnapshotError, snapshot::Snapshot};

mod cli;
mod config;
mod error;
mod fs;
mod snapshot;

fn main() -> Result<(), SnapshotError> {
    let snapshot = Snapshot::new()?;
    let output = snapshot.config.output_path.clone();

    snapshot.write_sources_md(&output)?;
    println!("✅ Project snapshot saved to {}", output.display());

    Ok(())
}
