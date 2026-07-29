use std::str::FromStr;

use crate::config::{MARKDOWN_EXTENSION, RUST_EXTENSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum OutputFormat {
    Rust,
    #[default]
    Markdown,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" | RUST_EXTENSION => Ok(OutputFormat::Rust),
            "markdown" | MARKDOWN_EXTENSION => Ok(OutputFormat::Markdown),
            _ => Err(format!(
                "Unknown format: {s}. Use 'rust', '{RUST_EXTENSION}', 'markdown', or '{MARKDOWN_EXTENSION}'"
            )),
        }
    }
}
