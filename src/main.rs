mod ast;
mod medusa_json;
mod parser;
mod reader;
mod types;
mod emitter;

use anyhow::Context;
use std::io::{self, IsTerminal, Read};
use std::fs::File;
use clap::Parser;

use crate::reader::Reader;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    file: Option<String>
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let args = Args::parse();

    let input: Box<dyn Read + 'static> = if !stdin.is_terminal() {
        Box::new(io::stdin())
    } else {
        // todo: error 
        Box::new(File::open(args.file.unwrap_or_default())?)
    };

    let reader = Reader::new(input);
    reader.parse().context("Error: Failed to parse") ?;

    Ok(())
}