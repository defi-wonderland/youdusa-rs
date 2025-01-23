mod ast;
mod emitter;
mod parser;
mod reader;
mod types;

use anyhow::Context;
use clap::Parser;
use std::fs::File;
use std::io::{self, IsTerminal, Read};

use crate::emitter::Emitter;
use crate::reader::Reader;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, help = "Optional text file to parse")]
    file: Option<String>,
}

/// Take a Medusa trace as input, parse it and create Foundry reproducer function for every failing properties
///
/// use either by piping the medusa process, `medusa fuzz | youdusa` either from a txt file, `youdusa --file log.txt`
fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let args = Args::parse();

    let input: Box<dyn Read + 'static> = if !stdin.is_terminal() {
        // piped input
        Box::new(io::stdin())
    } else {
        // file provided
        // todo: error instead of default (ie no pipe and no file)
        Box::new(File::open(args.file.unwrap_or_default())?)
    };

    // build the ast
    let reader = Reader::new(input);
    let ast = reader.parse().context("Error: Failed to parse")?;

    // emit the ast as solidity functions
    if let Some(ast_to_emit) = ast {
        for ast in ast_to_emit {
            let mut emitter = Emitter::new();
            emitter.emit(&ast).context("Error: Failed to create solidity function")?;
            println!("{}", emitter.get_emitted()); // for now, we stdout the reproducers
        }
    }

    Ok(())
}
