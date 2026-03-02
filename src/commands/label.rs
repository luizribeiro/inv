use std::path::Path;

use serde::Serialize;
use uuid::Uuid;

use crate::error::{AppError, Result};

#[derive(Debug, Serialize)]
struct LabelPayload {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bin_size: Option<String>,
    quantity: u64,
    unit: String,
}

pub fn run(db_path: &Path, id: &str, json: bool) -> Result<()> {
    let item_id = Uuid::parse_str(id)
        .map_err(|_| AppError::Validation(format!("invalid item id '{id}' (expected UUID)")))?;

    let doc = crate::storage::load_inventory(db_path)?;
    let item = doc
        .items
        .iter()
        .find(|candidate| candidate.id == item_id)
        .ok_or_else(|| AppError::Validation(format!("item '{item_id}' not found")))?;

    let payload = LabelPayload {
        id: item.id.as_hyphenated().to_string(),
        name: item.name.clone(),
        location: item.location.clone(),
        bin_size: item.bin_size.clone(),
        quantity: item.quantity,
        unit: item.unit.clone(),
    };

    if json {
        let output = serde_json::to_string_pretty(&payload).map_err(|error| {
            AppError::Validation(format!("failed to serialize label output: {error}"))
        })?;
        println!("{output}");
        return Ok(());
    }

    println!("Label placeholder (v1)");
    println!("id: {}", payload.id);
    println!("name: {}", payload.name);
    println!("quantity: {} {}", payload.quantity, payload.unit);
    println!(
        "location: {}",
        payload.location.as_deref().unwrap_or("(not set)")
    );
    println!(
        "bin_size: {}",
        payload.bin_size.as_deref().unwrap_or("(not set)")
    );

    Ok(())
}
