use std::io::Read;
use std::path::Path;

use dialoguer::Input;
use serde_json::Value;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::model::Item;

const UPDATE_TEST_INPUT_ENV: &str = "INV_UPDATE_TEST_INPUT";

#[derive(Debug, Clone, Default)]
struct UpdateInputData {
    name: Option<String>,
    description: Option<Option<String>>,
    quantity: Option<u64>,
    unit: Option<String>,
    location: Option<Option<String>>,
    bin_size: Option<Option<String>>,
    supplier: Option<Option<String>>,
    source_url: Option<Option<String>>,
    manufacturer: Option<Option<String>>,
    mpn: Option<Option<String>>,
    tags: Option<Vec<String>>,
    notes: Option<Option<String>>,
}

pub fn run(db_path: &Path, id: &str, stdin_json: bool) -> Result<()> {
    let item_id = Uuid::parse_str(id)
        .map_err(|_| AppError::Validation(format!("invalid item id '{id}' (expected UUID)")))?;

    let mut doc = crate::storage::load_inventory(db_path)?;
    let item_index = doc
        .items
        .iter()
        .position(|candidate| candidate.id == item_id)
        .ok_or_else(|| AppError::Validation(format!("item '{item_id}' not found")))?;

    let input = if stdin_json {
        collect_input_from_stdin()?
    } else {
        collect_input_interactive(&doc.items[item_index])?
    };
    let updated_item = apply_update(&doc.items[item_index], input)?;

    doc.items[item_index] = updated_item.clone();

    crate::model::validate_semantics(&doc)?;
    crate::storage::save_inventory_atomic(db_path, &doc)?;

    println!(
        "Updated item {}: {}",
        updated_item.id.as_hyphenated(),
        updated_item.name
    );

    Ok(())
}

fn collect_input_interactive(current: &Item) -> Result<UpdateInputData> {
    if let Ok(raw) = std::env::var(UPDATE_TEST_INPUT_ENV) {
        return parse_update_input_from_json(&raw).map_err(|error| {
            AppError::Validation(format!(
                "failed to parse {UPDATE_TEST_INPUT_ENV} as JSON update input: {error}"
            ))
        });
    }

    let name: String = Input::new()
        .with_prompt("Name")
        .default(current.name.clone())
        .allow_empty(true)
        .validate_with(|value: &String| -> std::result::Result<(), &str> {
            if value.trim().is_empty() {
                Err("name cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .map_err(|error| AppError::Validation(format!("failed to read Name input: {error}")))?;

    let description = prompt_optional_prefilled("Description", current.description.as_deref())?;
    let quantity = prompt_quantity_prefilled(current.quantity)?;
    let unit = prompt_default_prefilled("Unit", &current.unit)?;
    let location = prompt_optional_prefilled("Location", current.location.as_deref())?;
    let bin_size = prompt_optional_prefilled("Bin size", current.bin_size.as_deref())?;
    let supplier = prompt_optional_prefilled("Supplier", current.supplier.as_deref())?;
    let source_url = prompt_optional_prefilled("Source URL", current.source_url.as_deref())?;
    let manufacturer = prompt_optional_prefilled("Manufacturer", current.manufacturer.as_deref())?;
    let mpn = prompt_optional_prefilled("MPN", current.mpn.as_deref())?;
    let tags = prompt_tags_prefilled(&current.tags)?;
    let notes = prompt_optional_prefilled("Notes", current.notes.as_deref())?;

    Ok(UpdateInputData {
        name: Some(name),
        description: Some(description),
        quantity: Some(quantity),
        unit: Some(unit),
        location: Some(location),
        bin_size: Some(bin_size),
        supplier: Some(supplier),
        source_url: Some(source_url),
        manufacturer: Some(manufacturer),
        mpn: Some(mpn),
        tags: Some(tags),
        notes: Some(notes),
    })
}

fn collect_input_from_stdin() -> Result<UpdateInputData> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw).map_err(|error| {
        AppError::Validation(format!("failed to read stdin JSON input: {error}"))
    })?;

    if raw.trim().is_empty() {
        return Err(AppError::Validation(
            "stdin JSON input is empty; provide a JSON object".to_string(),
        ));
    }

    parse_update_input_from_json(&raw)
        .map_err(|error| AppError::Validation(format!("failed to parse stdin JSON input: {error}")))
}

fn parse_update_input_from_json(raw: &str) -> std::result::Result<UpdateInputData, String> {
    let value: Value = serde_json::from_str(raw).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "expected top-level JSON object".to_string())?;

    Ok(UpdateInputData {
        name: parse_optional_string_field(object, "name")?,
        description: parse_optional_nullable_string_field(object, "description")?,
        quantity: parse_optional_u64_field(object, "quantity")?,
        unit: parse_optional_string_field(object, "unit")?,
        location: parse_optional_nullable_string_field(object, "location")?,
        bin_size: parse_optional_nullable_string_field(object, "bin_size")?,
        supplier: parse_optional_nullable_string_field(object, "supplier")?,
        source_url: parse_optional_nullable_string_field(object, "source_url")?,
        manufacturer: parse_optional_nullable_string_field(object, "manufacturer")?,
        mpn: parse_optional_nullable_string_field(object, "mpn")?,
        tags: parse_optional_tags_field(object, "tags")?,
        notes: parse_optional_nullable_string_field(object, "notes")?,
    })
}

fn parse_optional_string_field(
    object: &serde_json::Map<String, Value>,
    field_name: &str,
) -> std::result::Result<Option<String>, String> {
    let Some(value) = object.get(field_name) else {
        return Ok(None);
    };

    match value {
        Value::String(inner) => Ok(Some(inner.clone())),
        _ => Err(format!(
            "field '{field_name}' must be a string when provided"
        )),
    }
}

fn parse_optional_nullable_string_field(
    object: &serde_json::Map<String, Value>,
    field_name: &str,
) -> std::result::Result<Option<Option<String>>, String> {
    let Some(value) = object.get(field_name) else {
        return Ok(None);
    };

    match value {
        Value::Null => Ok(Some(None)),
        Value::String(inner) => Ok(Some(Some(inner.clone()))),
        _ => Err(format!(
            "field '{field_name}' must be a string or null when provided"
        )),
    }
}

fn parse_optional_u64_field(
    object: &serde_json::Map<String, Value>,
    field_name: &str,
) -> std::result::Result<Option<u64>, String> {
    let Some(value) = object.get(field_name) else {
        return Ok(None);
    };

    match value {
        Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| format!("field '{field_name}' must be a non-negative integer"))
            .map(Some),
        _ => Err(format!(
            "field '{field_name}' must be a non-negative integer when provided"
        )),
    }
}

fn parse_optional_tags_field(
    object: &serde_json::Map<String, Value>,
    field_name: &str,
) -> std::result::Result<Option<Vec<String>>, String> {
    let Some(value) = object.get(field_name) else {
        return Ok(None);
    };

    let array = value
        .as_array()
        .ok_or_else(|| format!("field '{field_name}' must be an array of strings"))?;

    let mut tags = Vec::with_capacity(array.len());
    for entry in array {
        let tag = entry
            .as_str()
            .ok_or_else(|| format!("field '{field_name}' must contain only strings"))?;
        tags.push(tag.to_string());
    }

    Ok(Some(tags))
}

fn prompt_optional_prefilled(prompt: &str, current: Option<&str>) -> Result<Option<String>> {
    let default = current.unwrap_or("").to_string();
    let value: String = Input::new()
        .with_prompt(prompt)
        .default(default)
        .allow_empty(true)
        .interact_text()
        .map_err(|error| AppError::Validation(format!("failed to read {prompt} input: {error}")))?;

    Ok(normalize_optional(value))
}

fn prompt_default_prefilled(prompt: &str, current: &str) -> Result<String> {
    let value: String = Input::new()
        .with_prompt(prompt)
        .default(current.to_string())
        .allow_empty(true)
        .interact_text()
        .map_err(|error| AppError::Validation(format!("failed to read {prompt} input: {error}")))?;

    Ok(if value.trim().is_empty() {
        "pcs".to_string()
    } else {
        value.trim().to_string()
    })
}

fn prompt_quantity_prefilled(current: u64) -> Result<u64> {
    let value: String = Input::new()
        .with_prompt("Quantity")
        .default(current.to_string())
        .allow_empty(true)
        .interact_text()
        .map_err(|error| AppError::Validation(format!("failed to read Quantity input: {error}")))?;

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }

    trimmed.parse::<u64>().map_err(|_| {
        AppError::Validation(format!(
            "invalid quantity '{trimmed}' (expected a non-negative integer)"
        ))
    })
}

fn prompt_tags_prefilled(current: &[String]) -> Result<Vec<String>> {
    let default = current.join(", ");
    let raw: String = Input::new()
        .with_prompt("Tags (comma-separated)")
        .default(default)
        .allow_empty(true)
        .interact_text()
        .map_err(|error| {
            AppError::Validation(format!(
                "failed to read Tags (comma-separated) input: {error}"
            ))
        })?;

    Ok(parse_tags_csv(&raw))
}

fn apply_update(current: &Item, input: UpdateInputData) -> Result<Item> {
    let mut updated = current.clone();

    if let Some(name) = input.name {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation("name cannot be empty".to_string()));
        }
        updated.name = name;
    }

    if let Some(description) = input.description {
        updated.description = normalize_optional_from_option(description);
    }

    if let Some(quantity) = input.quantity {
        updated.quantity = quantity;
    }

    if let Some(unit) = input.unit {
        updated.unit = normalize_unit(Some(unit));
    }

    if let Some(location) = input.location {
        updated.location = normalize_optional_from_option(location);
    }

    if let Some(bin_size) = input.bin_size {
        updated.bin_size = normalize_optional_from_option(bin_size);
    }

    if let Some(supplier) = input.supplier {
        updated.supplier = normalize_optional_from_option(supplier);
    }

    if let Some(source_url) = input.source_url {
        updated.source_url = normalize_optional_from_option(source_url);
    }

    if let Some(manufacturer) = input.manufacturer {
        updated.manufacturer = normalize_optional_from_option(manufacturer);
    }

    if let Some(mpn) = input.mpn {
        updated.mpn = normalize_optional_from_option(mpn);
    }

    if let Some(tags) = input.tags {
        updated.tags = normalize_tags(Some(tags));
    }

    if let Some(notes) = input.notes {
        updated.notes = normalize_optional_from_option(notes);
    }

    updated.refresh_updated_at();

    Ok(updated)
}

fn normalize_unit(unit: Option<String>) -> String {
    unit.and_then(normalize_optional)
        .unwrap_or_else(|| "pcs".to_string())
}

fn normalize_tags(tags: Option<Vec<String>>) -> Vec<String> {
    tags.unwrap_or_default()
        .into_iter()
        .filter_map(normalize_optional)
        .collect()
}

fn normalize_optional_from_option(value: Option<String>) -> Option<String> {
    value.and_then(normalize_optional)
}

fn normalize_optional(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_tags_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .filter_map(|part| normalize_optional(part.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_apply_update_changes_selected_fields_and_normalizes_values() {
        let mut item = Item::with_required_fields(Uuid::new_v4(), "Original name");
        item.description = Some("Original description".to_string());
        item.quantity = 2;
        item.unit = "pcs".to_string();
        item.tags = vec!["old".to_string()];
        let original_created_at = item.created_at;
        let original_updated_at = item.updated_at;

        let updated = apply_update(
            &item,
            UpdateInputData {
                name: Some("  New name  ".to_string()),
                description: Some(Some("   ".to_string())),
                quantity: Some(10),
                unit: Some(" packs ".to_string()),
                location: Some(Some(" Drawer B ".to_string())),
                tags: Some(vec![" passive ".to_string(), " ".to_string()]),
                notes: Some(Some("  note  ".to_string())),
                ..UpdateInputData::default()
            },
        )
        .expect("update should succeed");

        assert_eq!(updated.name, "New name");
        assert_eq!(updated.description, None);
        assert_eq!(updated.quantity, 10);
        assert_eq!(updated.unit, "packs");
        assert_eq!(updated.location.as_deref(), Some("Drawer B"));
        assert_eq!(updated.tags, vec!["passive".to_string()]);
        assert_eq!(updated.notes.as_deref(), Some("note"));
        assert_eq!(updated.created_at, original_created_at);
        assert!(updated.updated_at >= original_updated_at);
    }

    #[test]
    fn update_apply_update_can_clear_optional_fields_with_null() {
        let mut item = Item::with_required_fields(Uuid::new_v4(), "Item");
        item.description = Some("Has description".to_string());
        item.location = Some("Shelf A".to_string());

        let updated = apply_update(
            &item,
            UpdateInputData {
                description: Some(None),
                location: Some(None),
                ..UpdateInputData::default()
            },
        )
        .expect("update should succeed");

        assert_eq!(updated.description, None);
        assert_eq!(updated.location, None);
    }

    #[test]
    fn update_apply_update_rejects_empty_name_when_provided() {
        let item = Item::with_required_fields(Uuid::new_v4(), "Item");

        let error = apply_update(
            &item,
            UpdateInputData {
                name: Some("   ".to_string()),
                ..UpdateInputData::default()
            },
        )
        .expect_err("empty name must fail");

        assert!(matches!(error, AppError::Validation(_)));
        assert!(error.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn update_collect_input_uses_test_hook_json() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(
            UPDATE_TEST_INPUT_ENV,
            r#"{"name":"Updated","description":null,"quantity":4,"tags":["a","b"]}"#,
        );

        let current = Item::with_required_fields(Uuid::new_v4(), "Current");
        let input = collect_input_interactive(&current).expect("test hook input should parse");

        assert_eq!(input.name.as_deref(), Some("Updated"));
        assert_eq!(input.description, Some(None));
        assert_eq!(input.quantity, Some(4));
        assert_eq!(input.tags, Some(vec!["a".to_string(), "b".to_string()]));

        std::env::remove_var(UPDATE_TEST_INPUT_ENV);
    }

    #[test]
    fn update_apply_update_keeps_existing_fields_when_input_omits_them() {
        let mut item = Item::with_required_fields(Uuid::new_v4(), "Item");
        item.quantity = 5;
        item.location = Some("Shelf C".to_string());
        let old_updated_at = item.updated_at;

        let updated = apply_update(&item, UpdateInputData::default()).expect("update should work");

        assert_eq!(updated.name, "Item");
        assert_eq!(updated.quantity, 5);
        assert_eq!(updated.location.as_deref(), Some("Shelf C"));
        assert!(updated.updated_at >= old_updated_at);
    }

    #[test]
    fn update_collect_input_prioritizes_stdin_json_over_test_env() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(UPDATE_TEST_INPUT_ENV, r#"{"name":"From env"}"#);

        let error = collect_input_from_stdin()
            .expect_err("empty stdin should fail when --stdin-json is set");

        assert!(error.to_string().contains("stdin JSON input is empty"));

        std::env::remove_var(UPDATE_TEST_INPUT_ENV);
    }
}
