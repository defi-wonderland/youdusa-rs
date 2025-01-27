use anyhow::Context;
use clap::{crate_authors, Parser};
use std::fs::File;
use std::io::{self, stdout, IsTerminal, Read};

#[derive(Parser)]
#[command(
    name = "youdusa",
    author = crate_authors!(",\n"),
    version,
    about,
    long_about = None,

    help_template = concat!(
include_str!("ascii_art.txt"),
"Made with ♥ by Wonderland (https://defi.sucks)\n
╔════════════════╗\n\
║    \x1B[31mYoudusa\x1B[0m     ║\n\
╚════════════════╝\n\
\n\
{about}\n\
\n\
{usage-heading} {usage}\n\
\n\
{all-args}\n\
\n\
Authors:{author-section}
Version: {version}
\n\
For more information, visit: https://github.com/defi-wonderland/youdusa-rs\n",
))]
struct Args {
    #[arg(
        short,
        long,
        help = "Optional text file to parse",
        long_help = "Specify a text file containing Medusa trace to parse. If not provided, \
                    the program will expect input from stdin."
    )]
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
