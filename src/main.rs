use anyhow::{Context, Result};

mod ast;
mod medusa_json;
mod parser;
mod runner;
mod types;

// TODO: take a file in instead (sometimes Medusa doesn't log a failing sequence
// before exiting, for unknown reason)

// TODO: handle sigint and exit gracefully (first Medusa, then write if needed, then Youdusa)

fn main() -> Result<()> {
    let path_to_entry = medusa_json::get_entry_point_path("medusa.json")
        .context("failed to process medusa config")?;

    runner::run(path_to_entry).context("Youdusa Error")?;

    Ok(())
}
