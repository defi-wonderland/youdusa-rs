use anyhow::Context;
use clap::Parser;
use std::fs::File;
use std::io::{self, stdout, IsTerminal, Read};

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
        match &args.file {
            Some(file) => Box::new(File::open(file).context("Failed to open input file")?),
            None => anyhow::bail!("No input provided. Either pipe input or use --file option"),
        }
    };

    let mut writer = stdout();
    youdusa::process_input(input, &mut writer).context("Youdusa failed")?;

    Ok(())
}
