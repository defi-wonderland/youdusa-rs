use crate::ast::{Ast, FunctionDeclaration, Statement};
use crate::emitter::Emitter;
use crate::types::CheatsData;

use anyhow::{Context, Ok, Result};
use std::collections::HashMap;

/// Define how to go from the raw stdio ouput to a complete ast
#[derive(Debug)]
pub struct Parser {
    /// Hashmap of the number of occurence a proprty fn has been seen
    unique_function_counter: HashMap<String, i32>,

    /// The current test function being filled
    current_ast_root: Option<Ast>,

    /// All the AST to emit later on
    reproducers: Vec<Ast>,
}

impl Parser {
    /// Branches out based on the line content ("FAILED" creates a new function,
    /// a numbered line is a new property function call, "Execution Trace" ends the
    /// current trace)
    pub fn process_line(&mut self, line: String) -> Result<()> {
        if line.contains("FAILED") {
            self.process_failed_assertion(&line);
        } else if line.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            self.add_new_call_to_ast(line)
                .context("failed to add new call to ast")?;
        } else if line.contains("[Execution Trace]") {
            if let Some(ast) = self.current_ast_root.take() {
                self.reproducers.push(ast);
            }
        }

        Ok(())
    }

    pub fn get_reproducers(self) -> Option<Vec<Ast>> {
        (!self.reproducers.is_empty()).then_some(self.reproducers)
    }

    pub fn new() -> Self {
        Self {
            unique_function_counter: HashMap::new(),
            current_ast_root: None,
            reproducers: Vec::new(),
        }
    }

    /// if we catch a "FAILED" keyword, start processing this failed assertion
    fn process_failed_assertion(&mut self, line: &str) {
        // Parse the FAILED line to get the property name then create a new AST
        if let Some(name) = self.extract_property_name(line) {
            let unique_name = self.get_unique_name(name);
            self.create_new_ast(unique_name);
        }
    }

    /// Isolate a property name from the rest of the line (only keep what comes after '.' and
    /// before '(' in "⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)")
    fn extract_property_name(&self, line: &str) -> Option<String> {
        line.split('.')
            .nth(1)?
            .split('(')
            .next()?
            .to_string()
            .into()
    }

    /// Add a number to a property name if it already exists plus track the number of occurences of this property
    /// (starting at 1 for a newly broken property)
    fn get_unique_name(&mut self, name: String) -> String {
        // self mut for entry()
        let counter = self
            .unique_function_counter
            .entry(name.clone())
            .or_insert(0);
        *counter += 1;

        if *counter > 1 {
            format!("test_{}{}", name, counter)
        } else {
            format!("test_{}", name)
        }
    }

    /// Start a new ast in temp current_ast
    fn create_new_ast(&mut self, name: String) {
        let new_fn = Ast::FunctionDeclaration(FunctionDeclaration::new(&name));
        self.current_ast_root = Some(new_fn);
    }

    /// Parse the line to extract the block height, msg sender, timestamp, fn name and its arguments, then
    /// add it as new children of the current temp ast (current_ast)
    /// example: "1) FuzzTest.property_canAlwaysCreateRequest(uint256,uint256)(1, 1) (block=43494, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)"
    fn add_new_call_to_ast(&mut self, line: String) -> Result<()> {
        // Parses out block, timestamp, sender, value (which we reuse later on)
        let cheats_data: CheatsData = self
            .parse_cheats_data(line.clone())
            .context("failed to parse call params")?;

        // Parses out: property_canAlwaysCreateRequest{value: 0}(1, 1)
        let property_call = self
            .parse_property_call(line.clone(), cheats_data.value)
            .context("failed to extract property to call")?;

        match &mut self.current_ast_root {
            Some(Ast::FunctionDeclaration(function_root)) => {
                function_root.add_child(Ast::Statement(Statement::new_roll(
                    cheats_data.block_to_roll,
                )));
                function_root.add_child(Ast::Statement(Statement::new_warp(
                    cheats_data.timestamp_to_warp_to,
                )));
                function_root.add_child(Ast::Statement(Statement::new_prank(
                    &cheats_data.caller_to_prank,
                )));
                function_root.add_child(Ast::Statement(property_call));
            }
            _ => return Err(anyhow::anyhow!("wrong parent")),
        }

        Ok(())
    }

    /// Parse the property name and create a new external call targeting 'this'
    fn parse_property_call(&self, line: String, value: i32) -> Result<Statement> {
        let property_name = self
            .extract_property_name(&line)
            .ok_or_else(|| anyhow::anyhow!("Failed to extract property name"))?;

        let arguments = self
            .parse_function_call_args(&line)
            .context("Failed to parse argsof property call")?;

        Ok(Statement::new_contract_call(
            Some("this".to_string()),
            property_name,
            Some(value),
            arguments,
        ))
    }

    /// Parse the values used in the different cheatcodes, as well as the msg.value to use
    /// They're all in the last '(..)', with a key=value format
    fn parse_cheats_data(&self, line: String) -> Option<CheatsData> {
        // First find the last '(..)' block
        line.rfind('(')
            .and_then(|start| line.rfind(')').map(|end| line[start + 1..end].to_string()))
            .and_then(|params| {
                // then by comma and collect relevant key-value pairs in a hashmap
                let pairs: Vec<(&str, &str)> = params
                    .split(',')
                    .filter_map(|pair| pair.split_once('=').map(|(k, v)| (k.trim(), v.trim())))
                    .collect();

                let map: std::collections::HashMap<_, _> = pairs.into_iter().collect();

                Some(CheatsData {
                    block_to_roll: map.get("block")?.parse().ok()?,
                    timestamp_to_warp_to: map.get("time")?.parse().ok()?,
                    caller_to_prank: map.get("sender")?.parse().ok()?,
                    value: map.get("value")?.parse().ok()?,
                })
            })
    }

    /// Parse the arguments of a given function call
    /// "property_foo(uint,uint,uint)(1, 2, 3)" returns \["1", "2", "3"\]
    /// this needs to handle hedge case like nested tuples/struct
    fn parse_function_call_args(&self, line: &str) -> Result<Vec<String>> {
        // TODO: we should parse and treat them as single element, including
        // nested tuples. For now, processed as a single block

        // discard the first half of parenthesis blocks, as these are the types
        let count = line.chars().filter(|c| *c == '(').count();
        let split_args = line
            .match_indices('(')
            .nth(count / 2)
            .map(|(index, _)| line.split_at(index));

        // empty bytes needs to be replaced with ''
        let values_fixed = split_args
            .unwrap_or_default()
            .1
            .to_string()
            .replace(",,", ",'',")
            .replace(",)", ",'')");

        Ok(vec![values_fixed])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_line_new_failure() {
        let mut parser = Parser::new();
        let test_line =
        "⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)";

        assert!(parser.process_line(test_line.to_string()).is_ok());
        assert_eq!(
            parser.current_ast_root,
            Some(Ast::FunctionDeclaration(FunctionDeclaration::new(
                "test_prop_anyoneCanIncreaseFundInAPool"
            )))
        );
    }

    ///@todo assert the content
    #[test]
    fn test_process_line_add_from_sequence() {
        let mut parser = Parser::new();
        let test_line =
        "1) FuzzTest.prop_alloOwnerCanAlwaysChangePercentFee(uint256)(15056796) (block=10429, time=19960, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000050000)";

        // We need a valid parent first
        parser.process_line("⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)".to_string()).expect("setup fail");

        assert!(parser.process_line(test_line.to_string()).is_ok());
    }

    #[test]
    fn test_extract_property_name_classic() {
        let parser = Parser::new();
        let test_line =
        "⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)";
        assert_eq!(
            parser.extract_property_name(test_line),
            Some("prop_anyoneCanIncreaseFundInAPool".to_string())
        );
    }

    #[test]
    fn test_extract_property_name_no_args() {
        let parser = Parser::new();
        let test_line = "⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool()";
        assert_eq!(
            parser.extract_property_name(test_line),
            Some("prop_anyoneCanIncreaseFundInAPool".to_string())
        );
    }

    #[test]
    fn test_extract_property_name_invalid_prop() {
        let parser = Parser::new();
        let test_line = "⇾ [FAILED] Assertion Test: prop_anyoneCanIncreaseFundInAPool()";
        assert_eq!(parser.extract_property_name(test_line), None);
    }

    #[test]
    fn test_get_unique_name() {
        let mut parser = Parser::new();
        let test_line = "prop_anyoneCanIncreaseFundInAPool";
        let _ = parser.get_unique_name(test_line.to_string());

        assert_eq!(
            parser.get_unique_name(test_line.to_string()),
            "test_prop_anyoneCanIncreaseFundInAPool2"
        );
    }

    #[test]
    fn test_get_unique_name_multiple() {
        let mut parser = Parser::new();
        let test_line = "prop_anyoneCanIncreaseFundInAPool";
        let _ = parser.get_unique_name(test_line.to_string());

        for i in 0..10 {
            assert_eq!(
                parser.get_unique_name(test_line.to_string()),
                format!("test_prop_anyoneCanIncreaseFundInAPool{}", i + 2)
            );
        }
    }

    #[test]
    fn test_get_unique_name_prop_with_number() {
        let mut parser = Parser::new();
        let test_line = "prop_anyoneCanIncreaseFundInAPool9";

        assert_eq!(
            parser.get_unique_name(test_line.to_string()),
            "test_prop_anyoneCanIncreaseFundInAPool9"
        );

        assert_eq!(
            parser.get_unique_name(test_line.to_string()),
            "test_prop_anyoneCanIncreaseFundInAPool92"
        );
    }

    #[test]
    fn test_parse_cheat_data() {
        let parser = Parser::new();
        let test_line = "1) FuzzTest.property_canAlwaysCreateRequest(uint256,uint256)(1, 1) (block=43494, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)";

        println!("{:?}", parser.parse_cheats_data(test_line.to_string()));

        // assert_eq!(
        //     runner.parse_cheats_data(test_line.to_string()),
        //     "prop_anyoneCanIncreaseFundInAPool9"
        // );
    }
}
