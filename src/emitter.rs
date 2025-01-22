/// Take an ast and create the corresponding solidity code
use anyhow::Result;
use crate::ast::{Ast, FunctionDeclaration, Statement, FunctionCall};


pub fn emit(ast: &Ast) -> Result<()> {
    match ast {
        Ast::FunctionDeclaration(fn_declaration) => emit_function(fn_declaration),
        Ast::Statement(statement) => emit_statement(statement) 
    }

    Ok(())
}

fn emit_function(fn_declaration: &FunctionDeclaration) {
    println!("function {}() public {{", fn_declaration.name());

    for child in fn_declaration.children() {
        match child {
            Ast::Statement(statement) => emit_statement(statement),
            Ast::FunctionDeclaration(fn_declaration) => emit_function(fn_declaration)
        }
    }

    println!("}}\n");
}

fn emit_statement(statement: &Statement) {
    match statement {
        Statement::ContractCall(contract_call) => emit_contract_call(contract_call)
    }
}

// this would be used for any fn call (internal or external)
fn emit_contract_call(contract_call: &FunctionCall) {
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
    call_to_construct.push_str(");");

    if add_new_line {
        call_to_construct.push('\n');
    }

    println!("{}", call_to_construct);
}