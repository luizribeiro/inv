use std::io::Read;
use std::path::Path;

use dialoguer::Input;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::model::Item;

const ADD_TEST_INPUT_ENV: &str = "INV_ADD_TEST_INPUT";

#[derive(Debug, Clone, Default, Deserialize)]
struct AddInputData {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    quantity: Option<u64>,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    bin_size: Option<String>,
    #[serde(default)]
    supplier: Option<String>,
    #[serde(default)]
    source_url: Option<String>,
    #[serde(default)]
    manufacturer: Option<String>,
    #[serde(default)]
    mpn: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    notes: Option<String>,
}

pub fn run(db_path: &Path, stdin_json: bool) -> Result<()> {
    let mut doc = crate::storage::load_inventory(db_path)?;
    let input = if stdin_json {
        collect_input_from_stdin()?
    } else {
        collect_input_interactive()?
    };

    let item = build_item_from_input(input)?;
    doc.items.push(item.clone());

    crate::model::validate_semantics(&doc)?;
    crate::storage::save_inventory_atomic(db_path, &doc)?;

    println!("Added item {}: {}", item.id.as_hyphenated(), item.name);

    Ok(())
}

fn collect_input_interactive() -> Result<AddInputData> {
    if let Ok(raw) = std::env::var(ADD_TEST_INPUT_ENV) {
        return serde_json::from_str(&raw).map_err(|error| {
            AppError::Validation(format!(
                "failed to parse {ADD_TEST_INPUT_ENV} as JSON add input: {error}"
            ))
        });
    }

    let name: String = Input::new()
        .with_prompt("Name")
        .validate_with(|value: &String| -> std::result::Result<(), &str> {
            if value.trim().is_empty() {
                Err("name cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()
        .map_err(|error| AppError::Validation(format!("failed to read name input: {error}")))?;

    let description = prompt_optional("Description")?;
    let quantity = prompt_quantity()?;
    let unit = prompt_default("Unit", "pcs")?;
    let location = prompt_optional("Location")?;
    let bin_size = prompt_optional("Bin size")?;
    let supplier = prompt_optional("Supplier")?;
    let source_url = prompt_optional("Source URL")?;
    let manufacturer = prompt_optional("Manufacturer")?;
    let mpn = prompt_optional("MPN")?;
    let tags = prompt_tags()?;
    let notes = prompt_optional("Notes")?;

    Ok(AddInputData {
        name,
        description,
        quantity: Some(quantity),
        unit: Some(unit),
        location,
        bin_size,
        supplier,
        source_url,
        manufacturer,
        mpn,
        tags: Some(tags),
        notes,
    })
}

fn collect_input_from_stdin() -> Result<AddInputData> {
    let mut raw = String::new();
    std::io::stdin().read_to_string(&mut raw).map_err(|error| {
        AppError::Validation(format!("failed to read stdin JSON input: {error}"))
    })?;

    if raw.trim().is_empty() {
        return Err(AppError::Validation(
            "stdin JSON input is empty; provide a JSON object".to_string(),
        ));
    }

    serde_json::from_str(&raw)
        .map_err(|error| AppError::Validation(format!("failed to parse stdin JSON input: {error}")))
}

fn prompt_optional(prompt: &str) -> Result<Option<String>> {
    let value: String = Input::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()
        .map_err(|error| AppError::Validation(format!("failed to read {prompt} input: {error}")))?;

    Ok(normalize_optional(value))
}

fn prompt_default(prompt: &str, default: &str) -> Result<String> {
    let value: String = Input::new()
        .with_prompt(prompt)
        .default(default.to_string())
        .allow_empty(true)
        .interact_text()
        .map_err(|error| AppError::Validation(format!("failed to read {prompt} input: {error}")))?;

    Ok(if value.trim().is_empty() {
        default.to_string()
    } else {
        value.trim().to_string()
    })
}

fn prompt_quantity() -> Result<u64> {
    let raw: String = Input::new()
        .with_prompt("Quantity")
        .default("0".to_string())
        .allow_empty(true)
        .interact_text()
        .map_err(|error| AppError::Validation(format!("failed to read Quantity input: {error}")))?;

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }

    trimmed.parse::<u64>().map_err(|_| {
        AppError::Validation(format!(
            "invalid quantity '{trimmed}' (expected a non-negative integer)"
        ))
    })
}

fn prompt_tags() -> Result<Vec<String>> {
    let raw: String = Input::new()
        .with_prompt("Tags (comma-separated)")
        .allow_empty(true)
        .interact_text()
        .map_err(|error| {
            AppError::Validation(format!(
                "failed to read Tags (comma-separated) input: {error}"
            ))
        })?;

    Ok(parse_tags_csv(&raw))
}

fn build_item_from_input(input: AddInputData) -> Result<Item> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("name cannot be empty".to_string()));
    }

    let mut item = Item::with_required_fields(Uuid::new_v4(), name);

    item.description = normalize_optional_from_option(input.description);
    item.quantity = input.quantity.unwrap_or(0);
    item.unit = normalize_unit(input.unit);
    item.location = normalize_optional_from_option(input.location);
    item.bin_size = normalize_optional_from_option(input.bin_size);
    item.supplier = normalize_optional_from_option(input.supplier);
    item.source_url = normalize_optional_from_option(input.source_url);
    item.manufacturer = normalize_optional_from_option(input.manufacturer);
    item.mpn = normalize_optional_from_option(input.mpn);
    item.tags = normalize_tags(input.tags);
    item.notes = normalize_optional_from_option(input.notes);

    Ok(item)
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
    fn add_build_item_uses_defaults_and_normalization() {
        let input = AddInputData {
            name: "  Resistor kit  ".to_string(),
            description: Some(" ".to_string()),
            quantity: None,
            unit: Some(" ".to_string()),
            location: Some(" Drawer A ".to_string()),
            bin_size: None,
            supplier: None,
            source_url: Some(" https://example.com/item ".to_string()),
            manufacturer: None,
            mpn: None,
            tags: Some(vec![
                "passive".to_string(),
                " ".to_string(),
                "smd".to_string(),
            ]),
            notes: Some("  note  ".to_string()),
        };

        let item = build_item_from_input(input).expect("build should succeed");

        assert_eq!(item.name, "Resistor kit");
        assert_eq!(item.quantity, 0);
        assert_eq!(item.unit, "pcs");
        assert_eq!(item.description, None);
        assert_eq!(item.location.as_deref(), Some("Drawer A"));
        assert_eq!(item.source_url.as_deref(), Some("https://example.com/item"));
        assert_eq!(item.tags, vec!["passive".to_string(), "smd".to_string()]);
        assert_eq!(item.notes.as_deref(), Some("note"));
    }

    #[test]
    fn add_build_item_rejects_empty_name() {
        let input = AddInputData {
            name: "   ".to_string(),
            ..AddInputData::default()
        };

        let error = build_item_from_input(input).expect_err("empty name must fail");
        assert!(matches!(error, AppError::Validation(_)));
        assert!(error.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn add_parse_tags_csv_trims_and_drops_empty_entries() {
        let tags = parse_tags_csv("  passive, , smd ,, through-hole  ");
        assert_eq!(tags, vec!["passive", "smd", "through-hole"]);
    }

    #[test]
    fn add_build_item_applies_explicit_values() {
        let input = AddInputData {
            name: "Capacitor kit".to_string(),
            description: Some("Ceramic assortment".to_string()),
            quantity: Some(12),
            unit: Some("packs".to_string()),
            location: Some("Shelf B".to_string()),
            bin_size: Some("medium".to_string()),
            supplier: Some("Mouser".to_string()),
            source_url: Some("https://example.com/caps".to_string()),
            manufacturer: Some("Generic".to_string()),
            mpn: Some("CAP-001".to_string()),
            tags: Some(vec!["passive".to_string()]),
            notes: Some("Restock soon".to_string()),
        };

        let item = build_item_from_input(input).expect("build should succeed");

        assert_eq!(item.quantity, 12);
        assert_eq!(item.unit, "packs");
        assert_eq!(item.bin_size.as_deref(), Some("medium"));
        assert_eq!(item.supplier.as_deref(), Some("Mouser"));
        assert_eq!(item.manufacturer.as_deref(), Some("Generic"));
        assert_eq!(item.mpn.as_deref(), Some("CAP-001"));
    }

    #[test]
    fn add_collect_input_uses_test_hook_json() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(
            ADD_TEST_INPUT_ENV,
            r#"{"name":"Test Item","quantity":2,"tags":["a","b"]}"#,
        );

        let input = collect_input_interactive().expect("test hook input should parse");

        assert_eq!(input.name, "Test Item");
        assert_eq!(input.quantity, Some(2));
        assert_eq!(input.tags, Some(vec!["a".to_string(), "b".to_string()]));

        std::env::remove_var(ADD_TEST_INPUT_ENV);
    }

    #[test]
    fn add_build_item_can_be_validated_and_persisted_in_doc() {
        let mut doc = crate::model::InventoryDoc::default();
        let item = build_item_from_input(AddInputData {
            name: "LM7805".to_string(),
            source_url: Some("https://example.com/lm7805".to_string()),
            ..AddInputData::default()
        })
        .expect("build should succeed");

        doc.items.push(item);
        crate::model::validate_semantics(&doc).expect("newly added item should validate");
    }

    #[test]
    fn add_collect_input_prioritizes_stdin_json_over_test_env() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(ADD_TEST_INPUT_ENV, r#"{"name":"From env"}"#);

        let input = collect_input_from_stdin()
            .expect_err("empty stdin should fail when --stdin-json is set");
        assert!(input.to_string().contains("stdin JSON input is empty"));

        std::env::remove_var(ADD_TEST_INPUT_ENV);
    }
}
