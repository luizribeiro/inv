use std::path::Path;

use crate::cli::Commands;
use crate::error::{AppError, Result};

pub fn run(command: &Commands, _db_path: &Path) -> Result<()> {
    Err(AppError::NotImplemented(command_name(command)))
}

fn command_name(command: &Commands) -> &'static str {
    match command {
        Commands::Init => "init",
        Commands::Add => "add",
        Commands::Update { .. } => "update",
        Commands::Search { .. } => "search",
        Commands::Show { .. } => "show",
        Commands::List { .. } => "list",
        Commands::Remove { .. } => "remove",
        Commands::Qr { .. } => "qr",
        Commands::Label { .. } => "label",
        Commands::Validate => "validate",
        Commands::IosSetup { .. } => "ios-setup",
    }
}
