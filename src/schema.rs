use std::sync::OnceLock;

use jsonschema::{Draft, JSONSchema};
use serde_json::Value;

use crate::error::{AppError, Result};
use crate::model::{self, InventoryDoc};

const INVENTORY_SCHEMA_JSON: &str = include_str!("../schema/inventory.schema.json");

#[allow(dead_code)]
pub fn validate_inventory(doc: &Value) -> Result<InventoryDoc> {
    let schema = compiled_schema()?;

    if let Err(errors) = schema.validate(doc) {
        let messages = errors
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(AppError::Validation(format!(
            "inventory schema validation failed: {messages}"
        )));
    }

    let parsed: InventoryDoc = serde_json::from_value(doc.clone()).map_err(|error| {
        AppError::Validation(format!(
            "inventory document deserialization failed: {error}"
        ))
    })?;

    model::validate_semantics(&parsed)?;

    Ok(parsed)
}

fn compiled_schema() -> Result<&'static JSONSchema> {
    static SCHEMA: OnceLock<Result<JSONSchema>> = OnceLock::new();

    SCHEMA
        .get_or_init(|| {
            let raw_schema: Value =
                serde_json::from_str(INVENTORY_SCHEMA_JSON).map_err(|error| {
                    AppError::Validation(format!(
                        "failed to parse embedded inventory schema: {error}"
                    ))
                })?;

            JSONSchema::options()
                .with_draft(Draft::Draft202012)
                .compile(&raw_schema)
                .map_err(|error| {
                    AppError::Validation(format!(
                        "failed to compile embedded inventory schema: {error}"
                    ))
                })
        })
        .as_ref()
        .map_err(|error| match error {
            AppError::Validation(message) => AppError::Validation(message.clone()),
            AppError::InvalidUrl { source, reason } => AppError::Validation(format!(
                "failed to compile embedded inventory schema from '{source}': {reason}"
            )),
            AppError::Io { path, action, source } => AppError::Validation(format!(
                "failed to compile embedded inventory schema while trying to {action} '{}': {source}",
                path.display()
            )),
            AppError::JsonParse { path, source } => AppError::Validation(format!(
                "failed to compile embedded inventory schema due to JSON parse error in '{}': {source}",
                path.display()
            )),
        })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::validate_inventory;

    fn valid_document() -> serde_json::Value {
        json!({
            "version": 1,
            "items": [
                {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "name": "ESP32 Dev Board",
                    "description": "Wi-Fi MCU",
                    "quantity": 2,
                    "unit": "pcs",
                    "location": "Shelf A",
                    "bin_size": "small",
                    "supplier": "LCSC",
                    "source_url": "https://example.com/esp32",
                    "manufacturer": "Espressif",
                    "mpn": "ESP32-WROOM-32",
                    "tags": ["mcu", "wifi"],
                    "notes": "Test note",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z"
                }
            ]
        })
    }

    #[test]
    fn valid_document_passes_schema_and_semantic_validation() {
        let document = valid_document();

        let parsed = validate_inventory(&document).expect("valid document must pass");

        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].name, "ESP32 Dev Board");
    }

    #[test]
    fn unknown_field_fails_schema_validation() {
        let mut document = valid_document();
        document["items"][0]["unexpected"] = json!("boom");

        let error = validate_inventory(&document).expect_err("unknown field must fail");

        let message = error.to_string();
        assert!(message.contains("schema validation failed"));
        assert!(message.contains("unexpected"));
    }

    #[test]
    fn malformed_shape_fails_schema_validation() {
        let document = json!({
            "version": 1,
            "items": [
                {
                    "id": "11111111-1111-1111-1111-111111111111"
                }
            ]
        });

        let error = validate_inventory(&document).expect_err("missing required field must fail");

        let message = error.to_string();
        assert!(message.contains("schema validation failed"));
        assert!(message.contains("name"));
    }

    #[test]
    fn semantic_validation_failures_are_reported_after_schema_validation() {
        let document = json!({
            "version": 1,
            "items": [
                {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "name": "First Item"
                },
                {
                    "id": "11111111-1111-1111-1111-111111111111",
                    "name": "Second Item"
                }
            ]
        });

        let error = validate_inventory(&document)
            .expect_err("duplicate ids should fail semantic validation");

        let message = error.to_string();
        assert!(message.contains("duplicate item id"));
    }
}
