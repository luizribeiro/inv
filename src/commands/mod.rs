use std::path::Path;

use crate::cli::Commands;
use crate::error::{AppError, Result};

mod add;
mod init;
mod list;
mod qr;
mod remove;
mod search;
mod show;
mod update;
mod validate;

pub fn run(command: &Commands, db_path: &Path) -> Result<()> {
    match command {
        Commands::Init => init::run(db_path),
        Commands::Add => add::run(db_path),
        Commands::Search { query, json } => search::run(db_path, query, *json),
        Commands::Show { id, json } => show::run(db_path, id, *json),
        Commands::List { json } => list::run(db_path, *json),
        Commands::Update { id } => update::run(db_path, id),
        Commands::Remove { id, yes } => remove::run(db_path, id, *yes),
        Commands::Qr { id, out } => qr::run(db_path, id, out.as_deref()),
        Commands::Validate => validate::run(db_path),
        _ => Err(AppError::NotImplemented(command_name(command))),
    }
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
