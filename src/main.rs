use anyhow::{Context, Result};

mod ast;
mod medusa_json;
mod runner;

fn main() -> Result<()> {
    let path_to_entry = medusa_json::get_entry_point_path("medusa.json")
        .context("failed to process medusa config")?;

    runner::run(path_to_entry)?;

    Ok(())
}
