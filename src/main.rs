mod app;
mod cli;
mod commands;
mod config;
mod error;
mod model;
mod schema;
mod storage;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    if let Err(err) = app::run(cli) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
