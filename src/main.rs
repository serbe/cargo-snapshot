use anyhow::Result;

use crate::project::Project;

mod config;
mod project;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    // let project = Project::default()?;
    let project = Project::new(std::path::Path::new(
        "D:\\forgejo\\xray-manager\\crates\\xray-daemon",
    ))?;
    // let project = Project::new(std::path::Path::new("D:\\forgejo\\xray-manager"))?;

    // print!("{project:?}");

    project.collect_sources("snapshot.rs")?;

    Ok(())
}
