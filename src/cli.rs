use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "inv", version, about = "Inventory as code CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init,
    Add,
    Update {
        id: String,
    },
    Search {
        query: String,
        #[arg(long)]
        json: bool,
    },
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Remove {
        id: String,
        #[arg(long)]
        yes: bool,
    },
    Qr {
        id: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Label {
        id: String,
        #[arg(long)]
        json: bool,
    },
    Validate,
    #[command(name = "ios-setup")]
    IosSetup {
        #[arg(long)]
        url: Option<String>,
    },
}
