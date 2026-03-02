mod cli;
mod commands;
mod error;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    if let Err(err) = commands::run(&cli.command) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
