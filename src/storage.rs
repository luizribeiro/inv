use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::error::{AppError, Result};
use crate::model::InventoryDoc;

#[allow(dead_code)]
pub fn load_inventory(path: &Path) -> Result<InventoryDoc> {
    let raw = fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        action: "read inventory file",
        source,
    })?;

    let document: Value = serde_json::from_str(&raw).map_err(|source| AppError::JsonParse {
        path: path.to_path_buf(),
        source,
    })?;

    crate::schema::validate_inventory(&document).map_err(|error| {
        AppError::Validation(format!("inventory file '{}': {error}", path.display()))
    })
}

#[cfg(test)]
mod tests {
    mod load {
        use std::fs;
        use std::io::ErrorKind;

        use tempfile::tempdir;

        use crate::error::AppError;
        use crate::storage::load_inventory;

        #[test]
        fn succeeds_for_well_formed_inventory_file() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("inventory.json");
            fs::write(&db_path, "{\n  \"version\": 1,\n  \"items\": []\n}\n")
                .expect("inventory file should be writable");

            let doc = load_inventory(&db_path).expect("valid inventory should load");

            assert_eq!(doc.version, 1);
            assert!(doc.items.is_empty());
        }

        #[test]
        fn reports_missing_file_with_context() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("does-not-exist.json");

            let error = load_inventory(&db_path).expect_err("missing file should fail");
            let message = error.to_string();

            assert!(message.contains("failed to read inventory file"));
            assert!(message.contains("does-not-exist.json"));

            match error {
                AppError::Io { source, .. } => assert_eq!(source.kind(), ErrorKind::NotFound),
                other => panic!("expected io error, got {other}"),
            }
        }

        #[test]
        fn reports_malformed_json_with_parse_location() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("inventory.json");
            fs::write(&db_path, "{\n  \"version\": 1,\n  \"items\": [\n")
                .expect("inventory file should be writable");

            let error = load_inventory(&db_path).expect_err("malformed json should fail");
            let message = error.to_string();

            assert!(message.contains("failed to parse JSON from"));
            assert!(message.contains("inventory.json"));
            assert!(message.contains("line") || message.contains("column"));
        }

        #[test]
        fn reports_schema_validation_with_file_context() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("inventory.json");
            fs::write(
                &db_path,
                "{\n  \"version\": 1,\n  \"items\": [{\n    \"id\": \"11111111-1111-1111-1111-111111111111\"\n  }]\n}\n",
            )
            .expect("inventory file should be writable");

            let error = load_inventory(&db_path).expect_err("schema-invalid json should fail");
            let message = error.to_string();

            assert!(message.contains("inventory file"));
            assert!(message.contains("inventory.json"));
            assert!(message.contains("schema validation failed"));
            assert!(message.contains("name"));
        }

        #[test]
        fn reports_semantic_validation_with_file_context() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("inventory.json");
            fs::write(
                &db_path,
                "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"First\"\n    },\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Second\"\n    }\n  ]\n}\n",
            )
            .expect("inventory file should be writable");

            let error = load_inventory(&db_path).expect_err("semantic-invalid json should fail");
            let message = error.to_string();

            assert!(message.contains("inventory file"));
            assert!(message.contains("inventory.json"));
            assert!(message.contains("duplicate item id"));
        }
    }
}
