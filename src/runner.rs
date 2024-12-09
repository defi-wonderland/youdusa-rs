use crate::parser::Parser;

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Create a new medusa process and returns the buffer reader
fn spawn_medusa_process(path: &PathBuf) -> Result<BufReader<std::process::ChildStdout>> {
    let mut child_process = Command::new("ping")
        .args(["-c", "4", "google.com"])
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to launch Medusa")?;

    let reader = BufReader::new(child_process.stdout.take().unwrap());
    Ok(reader)
}

/// Runs Medusa process
pub fn run(path: PathBuf) -> anyhow::Result<()> {
    let medusa_reader = spawn_medusa_process(&path).context("failed to launch medusa process")?;

    let mut parser = Parser::new();

    // todo: clippy complains flatten should be used (as only the Ok variant is used), but then,
    // lines can return an infinite streams of error (ie pathbuffer pointing to a dir etc, no realistic tho),
    // so flatten would inf loop on the first 'next' -> should use map_while(Ok), but todo?
    for line in medusa_reader.lines() {
        if let Ok(line) = line {
            println!("{}", line);
            parser
                .process_line(line)
                .context("failed to process line")?;
        }

        // If a complete function is pending, write it to the target contract
        parser.write_buffer_if_needed(&path).context("IO Error")?; // todo: add log "one test written to.."
    }

    // Probably print some stats here (x properties failing, writing x tests, etc)

    // todo: now this doesn't work anymore, would need to return Child instead of ChildStdOut from spawn_medusa_process,
    // -> check if Child::stdout is accessible -> is it really needed tho? We Ok(()) anyway
    // match medusa_reader.try_wait() {
    //     Ok(Some(status)) => println!("Medusa has finished with status: {}", status),
    //     Ok(None) => println!("Medusa has not finished yet"),
    //     Err(e) => println!("error attempting to wait: {}", e),
    // }

    Ok(())
}
