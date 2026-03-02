use std::path::Path;

use uuid::Uuid;

use crate::error::{AppError, Result};

pub fn run(db_path: &Path, id: &str, json: bool) -> Result<()> {
    let item_id = Uuid::parse_str(id)
        .map_err(|_| AppError::Validation(format!("invalid item id '{id}' (expected UUID)")))?;

    let doc = crate::storage::load_inventory(db_path)?;
    let item = doc
        .items
        .iter()
        .find(|candidate| candidate.id == item_id)
        .ok_or_else(|| AppError::Validation(format!("item '{item_id}' not found")))?;

    if json {
        let output = serde_json::to_string_pretty(item).map_err(|error| {
            AppError::Validation(format!("failed to serialize show output: {error}"))
        })?;
        println!("{output}");
        return Ok(());
    }

    println!("id: {}", item.id.as_hyphenated());
    println!("name: {}", item.name);
    println!("quantity: {} {}", item.quantity, item.unit);
    if let Some(description) = &item.description {
        println!("description: {description}");
    }
    if let Some(location) = &item.location {
        println!("location: {location}");
    }

    Ok(())
}
