use std::path::Path;

use crate::error::{AppError, Result};
use crate::model::Item;

pub fn run(db_path: &Path, query: &str, json: bool) -> Result<()> {
    let mut doc = crate::storage::load_inventory(db_path)?;
    doc.items
        .sort_by_key(|item| item.id.as_hyphenated().to_string());

    let original_query = query;
    let query = query.to_lowercase();
    let matches: Vec<Item> = doc
        .items
        .iter()
        .filter(|item| matches_query(item, &query))
        .cloned()
        .collect();

    if json {
        let output = serde_json::to_string_pretty(&matches).map_err(|error| {
            AppError::Validation(format!("failed to serialize search output: {error}"))
        })?;
        println!("{output}");
        return Ok(());
    }

    if matches.is_empty() {
        println!("No items matched query '{original_query}'.");
        return Ok(());
    }

    for item in &matches {
        println!(
            "{}\t{}\t{} {}",
            item.id.as_hyphenated(),
            item.name,
            item.quantity,
            item.unit
        );
    }

    Ok(())
}

fn matches_query(item: &Item, query: &str) -> bool {
    item.name.to_lowercase().contains(query)
        || item
            .description
            .as_ref()
            .is_some_and(|description| description.to_lowercase().contains(query))
}
