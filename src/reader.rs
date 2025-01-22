use crate::ast::{Ast, FunctionDeclaration, Statement};
use crate::emitter::Emitter;
use crate::parser::Parser;
use anyhow::Context;
use std::io::{BufRead, BufReader, Read};

pub struct Reader {
    buffer: BufReader<Box<dyn Read>>,
}

impl Reader {
    pub fn new(flux: Box<dyn Read>) -> Self {
        Self {
            buffer: BufReader::new(flux),
        }
    }

    pub fn parse(self) -> anyhow::Result<Option<Vec<Ast>>> {
        let mut parser = Parser::new();

        for line in self.buffer.lines() {
            if let Ok(line) = line {
                parser
                    .process_line(line)
                    .context("Error: Failed to process line")?;
            }
        }

        Ok(parser.get_reproducers())
    }
}
