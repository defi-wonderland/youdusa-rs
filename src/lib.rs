mod ast;
mod emitter;
mod parser;
mod reader;
mod types;

use anyhow::Context;

use std::io::{Read, Write};

use crate::emitter::Emitter;
use crate::reader::Reader;

/// Take a Medusa trace as input, parse it and create Foundry reproducer function for every failing properties
///
/// use either by piping the medusa process, `medusa fuzz | youdusa` either from a txt file, `youdusa --file log.txt`
pub fn process_input(
    input: Box<dyn Read + 'static>,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    // build the ast
    let reader = Reader::new(input);
    let ast = reader.parse().context("Error: Failed to parse")?;

    // emit the ast as solidity functions
    if let Some(ast_to_emit) = ast {
        for ast in ast_to_emit {
            let mut emitter = Emitter::new();
            emitter
                .emit(&ast)
                .context("Error: Failed to create solidity function")?;
            writeln!(writer, "{}", emitter.get_emitted())
                .context("Error: Failed to write reproducer")?;
        }
    }

    Ok(())
}
