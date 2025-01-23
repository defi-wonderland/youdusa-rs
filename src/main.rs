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
    #[arg(short, long)]
    file: Option<String>,
}

/// Take a Medusa trace as input, parse it and create Foundry
/// reproducer function for every failing properties
/// 
/// version: early mvp
fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let args = Args::parse();

    let input: Box<dyn Read + 'static> = if !stdin.is_terminal() {
        Box::new(io::stdin())
    } else {
        // todo: error instead of default
        Box::new(File::open(args.file.unwrap_or_default())?)
    };

    let reader = Reader::new(input);
    let ast = reader.parse().context("Error: Failed to parse")?;

    if let Some(ast_to_emit) = ast {
        for ast in ast_to_emit {
            let mut emitter = Emitter::new();
            emitter.emit(&ast)?;
            println!("{}", emitter.get_emitted()); // for now, we log the reproducers
        }
    }

    Ok(())
}
