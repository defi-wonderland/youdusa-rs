use crate::ast::{Ast, Function};
use crate::types::CheatsData;

use anyhow::{Context, Result};
// use std::io::{self, Write};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Wrap a medusa process
struct MedusaRunner {
    /// Hash map of the number of occurence a property has failed
    collected_fn: HashMap<String, i32>,

    /// Vec of the ast to append in the FuzzTest contract
    reproducers: Vec<Ast>,

    /// A temporary ast, used to build it (based on stdout) before pushing it
    /// to the reproducers vec
    current_ast: Option<Ast>,
}

impl MedusaRunner {
    fn new() -> Self {
        MedusaRunner {
            collected_fn: HashMap::new(),
            reproducers: Vec::new(),
            current_ast: None,
        }
    }

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

    /// if we catch a "FAILED" keyword, start processing this failed assertion
    fn process_failed_assertion(&mut self, line: &str) {
        // Parse the FAILED line to get the property name then create a new AST
        if let Some(name) = self.extract_property_name(line) {
            let unique_name = self.get_unique_name(name);
            self.create_new_ast(unique_name);
        }
    }

    /// Trim the property name of the rest of the line (only keep what comes after '.' and
    /// before '(' in "⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)")
    fn extract_property_name(&self, line: &str) -> Option<String> {
        line.split('.')
            .nth(1)?
            .split('(')
            .next()?
            .to_string()
            .into()
    }

    /// Add a number to the property name if it already exists plus track the number of occurences of this property
    /// (starting at 1 for a newly broken property)
    fn get_unique_name(&mut self, name: String) -> String {
        // self mut for entry()
        let counter = self.collected_fn.entry(name.clone()).or_insert(0);
        *counter += 1;

        if *counter > 1 {
            format!("{}{}", name, counter)
        } else {
            name
        }
    }

    /// Create a new ast and store it in the temp current_ast
    fn create_new_ast(&mut self, name: String) {
        let new_fn = Ast::Function(Function::new(&name));
        self.current_ast = Some(new_fn);
    }

    /// Push the temp ast to the reproducer ves
    fn finalize_ast(&mut self) {
        if let Some(ast) = self.current_ast.take() {
            self.reproducers.push(ast);
            self.current_ast = None;
        }

        // todo: we should write the test to the FuzzTest contract here (ie on the fly)
    }

    /// Parse the line to extract the block height, msg sender, timestamp, fn name and its arguments, then
    /// add it as new children of the current temp ast (current_ast)
    /// This should be pushed to the ast in the order property fn call, then cheatcodes (visitor being lifo)
    /// example: "1) FuzzTest.property_canAlwaysCreateRequest(uint256,uint256)(1, 1) (block=43494, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)"
    fn add_new_call_to_ast(&mut self, line: String) -> Result<()> {
        let cheatsData: CheatsData = self.parse_cheats_data(line).context("failed to parse call params")?;

        // Parse the block height
        // Push roll
        // Parse the msg sender
        // push prank
        // Parse the timestamp
        // push warp
        // Parse the fn name and its arguments
        // push call
        Ok(())
    }

    fn parse_cheats_data(&self, line: String) -> Option<CheatsData> {
        line.rfind('(')
            .and_then(|start| {
                line.rfind(')')
                    .map(|end| line[start+1..end].to_string())
            })
            .and_then(|params| {
                // Split by comma and collect key-value pairs
                let pairs: Vec<(&str, &str)> = params
                    .split(',')
                    .filter_map(|pair| {
                        pair.split_once('=')
                            .map(|(k, v)| (k.trim(), v.trim()))
                    })
                    .collect();

                // Create a HashMap for easier lookup
                let map: std::collections::HashMap<_, _> = pairs.into_iter().collect();

                // Parse values with proper error handling
                Some(CheatsData {
                    blockToRoll: map.get("block")?.parse().ok()?,
                    timestampToWarp: map.get("time")?.parse().ok()?,
                    senderToPrank: map.get("sender")?.parse().ok()?,
                    value: map.get("value")?.parse().ok()?,
                })
            })
    }

    fn pretty_print(&self, line: &str) {
        println!("{}", line);
    }

    fn process_line(&mut self, line: String) {
        self.pretty_print(&line);

        if line.contains("FAILED") {
            self.process_failed_assertion(&line);
        } else if line.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            self.add_new_call_to_ast(line);
        } else if line.contains("Execution Trace") {
            self.finalize_ast();
        }
    }
}

pub fn run(path: PathBuf) -> anyhow::Result<()> {
    let mut runner = MedusaRunner::new();
    let medusa_reader =
        MedusaRunner::spawn_medusa_process(&path).context("failed to launch medusa process")?;

    // todo: clippy complains flatten should be used (as only the Ok variant is used), but then,
    // lines can return an infinite streams of error (ie pathbuffer pointing to a dir etc, no realistic tho),
    // so flatten would inf loop on the first 'next' -> should use map_while(Ok), but todo?
    for line in medusa_reader.lines() {
        if let Ok(line) = line {
            runner.process_line(line);
        }
    }

    // todo: now this doesn't work anymore, would need to return Child instead of ChildStdOut from spawn_medusa_process,
    // -> check if Child::stdout is accessible -> is it really needed tho? We Ok(()) anyway
    // match runner.try_wait() {
    //     Ok(Some(status)) => println!("Medusa has finished with status: {}", status),
    //     Ok(None) => println!("Medusa has not finished yet"),
    //     Err(e) => println!("error attempting to wait: {}", e),
    // }

    Ok(())
}

#[cfg(test)]
#[test]
fn test_extract_property_name_classic() {
    let runner = MedusaRunner::new();
    let test_line =
        "⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)";
    assert_eq!(
        runner.extract_property_name(test_line),
        Some("prop_anyoneCanIncreaseFundInAPool".to_string())
    );
}

#[test]
fn test_extract_property_name_no_args() {
    let runner = MedusaRunner::new();
    let test_line = "⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool()";
    assert_eq!(
        runner.extract_property_name(test_line),
        Some("prop_anyoneCanIncreaseFundInAPool".to_string())
    );
}

#[test]
fn test_extract_property_name_invalid_prop() {
    let runner = MedusaRunner::new();
    let test_line = "⇾ [FAILED] Assertion Test: prop_anyoneCanIncreaseFundInAPool()";
    assert_eq!(runner.extract_property_name(test_line), None);
}

#[test]
fn test_get_unique_name() {
    let mut runner = MedusaRunner::new();
    let test_line = "prop_anyoneCanIncreaseFundInAPool";
    let _ = runner.get_unique_name(test_line.to_string());

    assert_eq!(
        runner.get_unique_name(test_line.to_string()),
        "prop_anyoneCanIncreaseFundInAPool2"
    );
}

#[test]
fn test_get_unique_name_multiple() {
    let mut runner = MedusaRunner::new();
    let test_line = "prop_anyoneCanIncreaseFundInAPool";
    let _ = runner.get_unique_name(test_line.to_string());

    for i in 0..10 {
        assert_eq!(
            runner.get_unique_name(test_line.to_string()),
            format!("prop_anyoneCanIncreaseFundInAPool{}", i + 2)
        );
    }
}

#[test]
fn test_get_unique_name_prop_with_number() {
    let mut runner = MedusaRunner::new();
    let test_line = "prop_anyoneCanIncreaseFundInAPool9";

    assert_eq!(
        runner.get_unique_name(test_line.to_string()),
        "prop_anyoneCanIncreaseFundInAPool9"
    );

    assert_eq!(
        runner.get_unique_name(test_line.to_string()),
        "prop_anyoneCanIncreaseFundInAPool92"
    );
}

#[test]
fn test_parse_cheat_data() {
    let mut runner = MedusaRunner::new();
    let test_line = "1) FuzzTest.property_canAlwaysCreateRequest(uint256,uint256)(1, 1) (block=43494, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)";


    println!("{:?}", runner.parse_cheats_data(test_line.to_string()));

    // assert_eq!(
    //     runner.parse_cheats_data(test_line.to_string()),
    //     "prop_anyoneCanIncreaseFundInAPool9"
    // );

}
