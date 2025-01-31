use crate::ast::{Ast, FunctionDeclaration, Statement};
use crate::types::CheatsData;

use anyhow::{anyhow, Context, Ok, Result};
use std::collections::HashMap;

/// Define how to go from the Medusa trace to a complete Youdusa ast
#[derive(Debug)]
pub struct Parser {
    /// Hashmap of the number of occurence a proprty fn has been seen
    /// @dev This is used to add numbering if a same property fails multiple times
    unique_function_counter: HashMap<String, i32>,

    /// The current solidity test function being build
    current_ast_root: Option<Ast>,

    /// All the ast already produced and finished
    reproducers: Vec<Ast>,
}

impl Parser {
    /// Branches out based on the line content:
    /// "FAILED" creates a ast (new property to reproduce, with correct naming),
    /// a numbered line is a new property function call (should be included as a new call)
    /// "Execution Trace" ends the current trace (push the current ast with the finished ones)
    pub fn process_line(&mut self, line: String) -> Result<()> {
        if line.contains("FAILED") {
            self.create_new_reproducer(&line)
                .context("failed to parse new broken property")?;
        } else if line.chars().next().map(|c| c.is_numeric()).unwrap_or(false)
            && self.current_ast_root.is_some()
        {
            self.add_new_call_to_ast(line)
                .context("failed to add new call to ast")?;
        } else if line.contains("[Execution Trace]") {
            if let Some(ast) = self.current_ast_root.take() {
                self.reproducers.push(ast);
            }
        }

        Ok(())
    }

    /// Return all the reproducer ast already built
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

    /// Start processing a new failed assertion, as a new ast
    fn create_new_reproducer(&mut self, line: &str) -> Result<()> {
        let name = self
            .extract_property_name(line)
            .ok_or_else(|| anyhow!("Couldn't parse property name"))?;
        let unique_name = self.generate_unique_test_name(name);
        self.create_new_ast(unique_name);
        Ok(())
    }

    /// Isolate a property name from the rest of the line
    /// only keep what comes after 'FuzzTest.' and before '(...' in
    /// ⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)
    fn extract_property_name(&self, line: &str) -> Option<String> {
        line.split('.')
            .nth(1)?
            .split('(')
            .next()?
            .to_string()
            .into()
    }

    /// Add a "test" prefix and a number suffix to a property name
    /// and track the number of occurences of this property
    fn generate_unique_test_name(&mut self, name: String) -> String {
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

    /// Start building a new ast
    fn create_new_ast(&mut self, name: String) {
        let new_fn = Ast::FunctionDeclaration(FunctionDeclaration::new(&name));
        self.current_ast_root = Some(new_fn);
    }

    /// Parse the line to extract block height, msg sender, timestamp, fn name and its arguments
    /// add them as new children of the current temp ast (as cheat codes or call to "this")
    fn add_new_call_to_ast(&mut self, line: String) -> Result<()> {
        // Parses out block, timestamp, sender, value (which we reuse later on)
        let cheats_data: CheatsData = self
            .parse_cheats_data(line.clone())
            .context("failed to parse call context")?;

        // Parses property_canAlwaysCreateRequest{value: 0}(1, 1)
        let property_call = self
            .generate_call_to_medusa_property(line.clone(), cheats_data.value)
            .context("failed to extract property to call")?;

        // Add all cheatcodes then the Medusa property to call
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
    /// @dev For now, the args are returned as a Vec containing a single String
    /// futureproof would be parse them individually, including nested struct
    fn generate_call_to_medusa_property(&self, line: String, value: i32) -> Result<Statement> {
        let property_name = self
            .extract_property_name(&line)
            .ok_or_else(|| anyhow::anyhow!("Failed to extract property name"))?;

        let arguments = self
            .parse_medusa_call_arguments(&line)
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
    /// "property_foo(uint,uint,uint)(1, 2, 3, (4, 5), )" returns "1, 2, 3, (4, 5), ''"
    /// this needs to handle hedge case like nested tuples/struct
    /// @dev Medusa returns an empty char for empty bytes, we replace it with ''
    /// @dev Temp(?), all args are parsed as a single String (easier to handler nested tuples)
    fn parse_medusa_call_arguments(&self, line: &str) -> Result<Vec<String>> {
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
            .replace(",)", ",'')")
            .replace("(,", "('',");

        // Remove the last parenthesis block (the cheatcodes)
        let res = values_fixed
            .rfind(" (")
            .map_or(values_fixed.as_str(), |i| &values_fixed[..i])
            .to_string();

        // Trim the outside parenthesis, added when emitting (ie easy transition to individual elements)
        Ok(vec![res
            .trim_start_matches("(")
            .trim_end_matches(")")
            .to_string()])
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

        let result = parser.process_line(test_line.to_string());

        assert!(result.is_ok());
        assert_eq!(
            parser.current_ast_root,
            Some(Ast::FunctionDeclaration(FunctionDeclaration::new(
                "test_prop_anyoneCanIncreaseFundInAPool"
            )))
        );
    }

    #[test]
    fn test_process_line_only_new_failure() {
        let mut parser = Parser::new();
        let test_line = "⇾ [FAIL] Foo";

        let result = parser.process_line(test_line.to_string());

        assert!(result.is_ok());
        assert_eq!(parser.current_ast_root, None);
    }

    #[test]
    fn test_process_line_fails_invalid_property_name() {
        let mut parser = Parser::new();
        let test_line = "⇾ [FAILED] Foo";

        let result = parser.process_line(test_line.to_string());

        assert_eq!(
            result.unwrap_err().to_string(),
            "failed to parse new broken property"
        );
        assert_eq!(parser.current_ast_root, None);
    }

    ///@todo assert the content
    #[test]
    fn test_process_line_add_from_sequence() {
        let mut parser = Parser::new();
        let test_line =
        "1) FuzzTest.prop_alloOwnerCanAlwaysChangePercentFee(uint256)(15056796) (block=10429, time=19960, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000050000)";

        // We need a valid parent first
        parser.process_line("⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)".to_string()).expect("setup fail");

        let result = parser.process_line(test_line.to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn test_process_line_end_sequence() {
        let mut parser = Parser::new();
        let test_line = "[Execution Trace]";

        // valid parent
        assert!(parser.process_line("⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)".to_string()).is_ok());

        let result = parser.process_line(test_line.to_string());

        assert!(result.is_ok());
        assert_eq!(
            parser.reproducers,
            vec![Ast::FunctionDeclaration(FunctionDeclaration::new(
                "test_prop_anyoneCanIncreaseFundInAPool"
            ))]
        );
        assert_eq!(parser.current_ast_root, None);
    }

    #[test]
    fn test_create_new_ast_root_creates() {
        let mut parser = Parser::new();
        let test_line =
            "⇾ [FAILED] Assertion Test: FuzzTest.prop_anyoneCanIncreaseFundInAPool(uint256,uint256)";

        let result = parser.create_new_reproducer(test_line);

        assert!(result.is_ok());

        assert_eq!(
            parser.current_ast_root,
            Some(Ast::FunctionDeclaration(FunctionDeclaration::new(
                "test_prop_anyoneCanIncreaseFundInAPool"
            )))
        );
    }

    #[test]
    fn test_create_new_ast_root_wrong_format() {
        let mut parser = Parser::new();
        let test_line = "⇾ [FAILED] Assertion Test: foo";

        let result = parser.create_new_reproducer(test_line);

        assert_eq!(
            result.unwrap_err().to_string(),
            "Couldn't parse property name"
        );
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
    fn test_generate_unique_test_name() {
        let mut parser = Parser::new();
        let test_line = "prop_anyoneCanIncreaseFundInAPool";

        assert_eq!(
            parser.generate_unique_test_name(test_line.to_string()),
            "test_prop_anyoneCanIncreaseFundInAPool"
        );
    }

    #[test]
    fn test_generate_unique_test_name_multiple() {
        let mut parser = Parser::new();
        let test_line = "prop_anyoneCanIncreaseFundInAPool";
        let _ = parser.generate_unique_test_name(test_line.to_string());

        for i in 0..10 {
            assert_eq!(
                parser.generate_unique_test_name(test_line.to_string()),
                format!("test_prop_anyoneCanIncreaseFundInAPool{}", i + 2)
            );
        }
    }

    #[test]
    fn test_generate_unique_test_name_prop_with_number() {
        let mut parser = Parser::new();
        let test_line = "prop_anyoneCanIncreaseFundInAPool9";

        assert_eq!(
            parser.generate_unique_test_name(test_line.to_string()),
            "test_prop_anyoneCanIncreaseFundInAPool9"
        );

        assert_eq!(
            parser.generate_unique_test_name(test_line.to_string()),
            "test_prop_anyoneCanIncreaseFundInAPool92"
        );
    }

    #[test]
    fn test_add_new_call_to_ast() {
        let mut parser = Parser::new();
        let test_line = "1) FuzzTest.property_canAlwaysCreateRequest(uint256,uint256)(1, 1) (block=43494, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)";

        parser.create_new_ast("test".to_string());

        let result = parser.add_new_call_to_ast(test_line.to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_new_call_to_ast_wrong_cheats() {
        let mut parser = Parser::new();
        let test_line = "1) FuzzTest.property_canAlwaysCreateRequest(uint256,uint256)(1, 1) (block=, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)";

        parser.create_new_ast("test".to_string());

        let result = parser.add_new_call_to_ast(test_line.to_string());

        assert_eq!(
            result.unwrap_err().to_string(),
            "failed to parse call context"
        );
    }

    #[test]
    fn test_add_new_call_to_as_wrong_args() {
        let mut parser = Parser::new();
        let test_line = "1) property_canAlwaysCreateRequest(uint256,uint256)(1, 1) (block=43494, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)";

        parser.create_new_ast("test".to_string());

        let result = parser.add_new_call_to_ast(test_line.to_string());

        assert_eq!(
            result.unwrap_err().to_string(),
            "failed to extract property to call"
        );
    }

    //@todo generate_call_to_medusa_property and parse_cheats_data

    #[test]
    fn test_parse_medusa_call_arguments() {
        let parser = Parser::new();
        let test_line = "1) property_canAlwaysCreateRequest(uint256,uint256)(1,1) (block=43494, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)";

        let result = parser.parse_medusa_call_arguments(test_line);

        assert!(result.is_ok());

        assert_eq!(result.unwrap()[0], "1,1");
    }

    #[test]
    fn test_parse_medusa_call_arguments_tuple() {
        let parser = Parser::new();
        let test_line = "1) property_canAlwaysCreateRequest(uint256,uint256,(address,uint256),address)(1,1,(0x12,1),0x12) (block=43494, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)";

        let result = parser.parse_medusa_call_arguments(test_line);

        assert!(result.is_ok());

        assert_eq!(result.unwrap()[0], "1,1,(0x12,1),0x12");
    }

    #[test]
    fn test_parse_medusa_call_arguments_bytes() {
        let parser = Parser::new();
        let test_line = "1) property_canAlwaysCreateRequest(uint256,bytes,(bytes,bytes,bytes),bytes)(1,,(,,),) (block=43494, time=315910, gas=12500000, gasprice=1, value=0, sender=0x0000000000000000000000000000000000060000)";

        let result = parser.parse_medusa_call_arguments(test_line);

        assert!(result.is_ok());

        assert_eq!(result.unwrap()[0], "1,'',('','',''),''");
    }
}
