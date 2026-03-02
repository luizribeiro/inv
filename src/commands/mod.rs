use std::path::Path;

use crate::cli::Commands;
use crate::error::Result;

mod add;
mod init;
mod ios_setup;
mod label;
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
        Commands::Add { stdin_json } => add::run(db_path, *stdin_json),
        Commands::Search { query, json } => search::run(db_path, query, *json),
        Commands::Show { id, json } => show::run(db_path, id, *json),
        Commands::List { json } => list::run(db_path, *json),
        Commands::Update { id } => update::run(db_path, id),
        Commands::Remove { id, yes } => remove::run(db_path, id, *yes),
        Commands::Qr { id, out } => qr::run(db_path, id, out.as_deref()),
        Commands::Label { id, json } => label::run(db_path, id, *json),
        Commands::Validate => validate::run(db_path),
        Commands::IosSetup { url } => ios_setup::run(url.clone()),
    }
}
