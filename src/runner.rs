use crate::ast;
use anyhow::Context;
// use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn run(path: PathBuf) -> anyhow::Result<()> {
    let mut child_process = Command::new("ping")
        .args(["-c", "4", "google.com"])
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to launch Medusa")?;

    let reader = BufReader::new(child_process.stdout.take().unwrap());

    let mut fn_to_append: Vec<ast::Ast> = Vec::new();

    reader.lines().for_each(|line| {
        let val = line.unwrap();
        println!("{}", val); // Todo: prettify Medusa output

        // Look for a failed assertion, to get the property name
        if val.contains("FAILED") {
            // Parse the FAILED line to get the property name
            // we parse "⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)";
            let name = val
                .split('.')
                .nth(1)
                .unwrap_or("")
                .split('(')
                .next()
                .unwrap_or("");

            // Check if there is a previous occurence of this property, add number if so

            // create a new fn to append, with the property name
        }

        // The call sequence is numbered, use this to isolate it
        if val.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {

            // Parse the block height
            // Parse the msg sender
            // Parse the timestamp
            // Parse the fn name and its arguments
            // Generate the function calls (cheatcode and property)
        }

        // End of this call sequence
        if val.contains("Execution Trace") {}
    });

    match child_process.try_wait() {
        Ok(Some(status)) => println!("child has finished with status: {}", status),
        Ok(None) => println!("child has not finished yet"),
        Err(e) => println!("error attempting to wait: {}", e),
    }

    Ok(())
}

#[cfg(test)]
#[test]
fn test_run() {}
