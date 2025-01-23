use crate::ast::{Ast, FunctionCall, FunctionDeclaration, Statement};
use anyhow::Result;

/// Take an ast and create the corresponding solidity code
pub struct Emitter {
    /// All the reproducer function ready to output
    output: String,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    /// Emit the solidity code from an Youdusa ast
    pub fn emit(&mut self, ast: &Ast) -> Result<()> {
        match ast {
            Ast::FunctionDeclaration(fn_declaration) => {
                self.emit_function_declaration(fn_declaration)
            }
            Ast::Statement(statement) => self.emit_statement(statement),
        }

        Ok(())
    }

    pub fn get_emitted(self) -> String {
        self.output
    }

    /// Emit a function declaration
    fn emit_function_declaration(&mut self, fn_declaration: &FunctionDeclaration) {
        self.output
            .push_str(&format!("function {}() public {{\n", fn_declaration.name()));

        // Add all the elements in the function body
        for child in fn_declaration.children() {
            match child {
                Ast::Statement(statement) => self.emit_statement(statement),
                Ast::FunctionDeclaration(fn_declaration) => {
                    self.emit_function_declaration(fn_declaration)
                }
            }
        }

        self.output.push_str("}}\n");
    }

    fn emit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::ContractCall(contract_call) => self.emit_contract_call(contract_call),
        }
    }

    /// Emit a call (used for any internal or external function call)
    /// `target.foo{ value: X }(a, b, c);`
    fn emit_contract_call(&mut self, contract_call: &FunctionCall) {
        let mut call_to_construct = String::new();
        let mut add_new_line = false; // only add a new line after an external call

        // Indent at current block level
        call_to_construct.push_str(&" ".repeat(4));

        // If external call, add the target
        if let Some(to_call) = &contract_call.external_contract {
            call_to_construct.push_str(to_call.as_str());
            call_to_construct.push('.');

            if to_call.contains("this") {
                add_new_line = true;
            }
        }

        // Add the function call
        call_to_construct.push_str(&contract_call.function_name);

        // Add a { value: X } if needed
        if let Some(value) = &contract_call.value {
            call_to_construct.push_str(&format!("{{ value: {} }}", value));
        }

        // Add all arguments
        call_to_construct.push('(');
        call_to_construct.push_str(&contract_call.arguments.join(", "));
        call_to_construct.push_str(");\n");

        // New line if it was an external call
        if add_new_line {
            call_to_construct.push('\n');
        }

        self.output.push_str(&call_to_construct);
    }
}
