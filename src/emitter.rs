use crate::ast::{Ast, FunctionCall, FunctionDeclaration, Statement};
/// Take an ast and create the corresponding solidity code
use anyhow::Result;

pub struct Emitter {
    output: String,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn emit(&mut self, ast: &Ast) -> Result<()> {
        match ast {
            Ast::FunctionDeclaration(fn_declaration) => self.emit_function(fn_declaration),
            Ast::Statement(statement) => self.emit_statement(statement),
        }

        Ok(())
    }

    pub fn get_emitted(self) -> String {
        self.output
    }

    fn emit_function(&mut self, fn_declaration: &FunctionDeclaration) {
        self.output
            .push_str(&format!("function {}() public {{\n", fn_declaration.name()));

        for child in fn_declaration.children() {
            match child {
                Ast::Statement(statement) => self.emit_statement(statement),
                Ast::FunctionDeclaration(fn_declaration) => self.emit_function(fn_declaration),
            }
        }

        self.output.push_str("}}\n");
    }

    fn emit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::ContractCall(contract_call) => self.emit_contract_call(contract_call),
        }
    }

    // this would be used for any fn call (internal or external)
    fn emit_contract_call(&mut self, contract_call: &FunctionCall) {
        let mut call_to_construct = String::new();
        let mut add_new_line = false;

        call_to_construct.push_str(&" ".repeat(4));

        if let Some(to_call) = &contract_call.external_contract {
            call_to_construct.push_str(to_call.as_str());
            call_to_construct.push('.');

            if to_call.contains("this") {
                add_new_line = true;
            }
        }

        call_to_construct.push_str(&contract_call.function_name);

        if let Some(value) = &contract_call.value {
            call_to_construct.push_str(&format!("{{ value: {} }}", value));
        }

        call_to_construct.push('(');
        call_to_construct.push_str(&contract_call.arguments.join(", "));
        call_to_construct.push_str(");\n");

        if add_new_line {
            call_to_construct.push('\n');
        }

        self.output.push_str(&call_to_construct);
    }
}
