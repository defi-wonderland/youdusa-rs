use crate::ast::Ast;
use crate::parser::Parser;
use anyhow::Context;
use std::io::{BufRead, BufReader, Read};

/// Wrapper around input sources implementing Read trait
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

        for line in self.buffer.lines().map_while(Result::ok) {
            parser
                .process_line(line)
                .context("Error: Failed to process line")?;
        }

        Ok(parser.get_reproducers())
    }
}
