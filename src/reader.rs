use crate::parser::Parser;

use anyhow::{Context};
use std::io::{BufRead, BufReader, Read};

pub struct Reader {
    buffer: BufReader<Box<dyn Read>>
}

impl Reader {
    pub fn new(flux: Box<dyn Read>) -> Self {
        Self {
            buffer: BufReader::new(flux)
        }
    }

    pub fn parse(self) -> anyhow::Result<()> {
        let mut parser = Parser::new();

        for line in self.buffer.lines() {
            if let Ok(line) = line {
                parser
                    .process_line(line)
                    .context("Error: Failed to process line")?;
            }

            // If a complete function is pending, write it to the target contract
            parser.write_buffer_if_needed().context("IO Error")?; // todo: add log "one test written to.."
        }
    Ok(())
    }
}