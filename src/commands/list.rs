use std::path::Path;

use crate::error::{AppError, Result};

pub fn run(db_path: &Path, json: bool) -> Result<()> {
    let mut doc = crate::storage::load_inventory(db_path)?;
    doc.items
        .sort_by_key(|item| item.id.as_hyphenated().to_string());

    if json {
        let output = serde_json::to_string_pretty(&doc.items).map_err(|error| {
            AppError::Validation(format!("failed to serialize list output: {error}"))
        })?;
        println!("{output}");
        return Ok(());
    }

    if doc.items.is_empty() {
        println!("No items found.");
        return Ok(());
    }

    for item in &doc.items {
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
