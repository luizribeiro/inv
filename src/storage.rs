use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

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

#[allow(dead_code)]
pub fn save_inventory_atomic(path: &Path, doc: &InventoryDoc) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    let mut sorted = doc.clone();
    sorted
        .items
        .sort_by_key(|item| item.id.as_hyphenated().to_string());

    let mut json = serde_json::to_string_pretty(&sorted).map_err(|error| {
        AppError::Validation(format!(
            "failed to serialize inventory document for saving: {error}"
        ))
    })?;
    json.push('\n');

    let temp_path = temp_path_for(path);

    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
    {
        Ok(file) => file,
        Err(source) => {
            return Err(AppError::Io {
                path: temp_path,
                action: "create temporary inventory file",
                source,
            });
        }
    };

    if let Err(source) = file.write_all(json.as_bytes()) {
        let _ = fs::remove_file(&temp_path);
        return Err(AppError::Io {
            path: temp_path,
            action: "write temporary inventory file",
            source,
        });
    }

    if let Err(source) = file.sync_all() {
        let _ = fs::remove_file(&temp_path);
        return Err(AppError::Io {
            path: temp_path,
            action: "fsync temporary inventory file",
            source,
        });
    }

    drop(file);

    if let Err(source) = rename_into_place(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(AppError::Io {
            path: path.to_path_buf(),
            action: "rename temporary inventory file into place",
            source,
        });
    }

    // Best-effort directory sync to make rename durable on platforms that support it.
    if let Ok(dir) = OpenOptions::new().read(true).open(parent) {
        let _ = dir.sync_all();
    }

    Ok(())
}

fn rename_into_place(temp_path: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(temp_path, destination)
}

fn temp_path_for(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("inventory.json");
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);

    parent.join(format!(".{file_name}.tmp.{}.{}", std::process::id(), nanos))
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

    mod save {
        use std::fs;
        use std::io::ErrorKind;

        use tempfile::tempdir;
        use uuid::Uuid;

        use crate::error::AppError;
        use crate::model::{InventoryDoc, Item};
        use crate::storage::{load_inventory, save_inventory_atomic};

        #[test]
        fn roundtrip_save_and_load_inventory() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("inventory.json");

            let mut item = Item::with_required_fields(Uuid::new_v4(), "Oscilloscope");
            item.quantity = 1;
            let doc = InventoryDoc {
                version: 1,
                items: vec![item.clone()],
            };

            save_inventory_atomic(&db_path, &doc).expect("save should succeed");
            let loaded = load_inventory(&db_path).expect("load after save should succeed");

            assert_eq!(loaded.version, 1);
            assert_eq!(loaded.items.len(), 1);
            assert_eq!(loaded.items[0].id, item.id);
            assert_eq!(loaded.items[0].name, "Oscilloscope");
            assert_eq!(loaded.items[0].quantity, 1);
        }

        #[test]
        fn save_orders_items_by_uuid_for_deterministic_output() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("inventory.json");

            let id_low =
                Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("valid uuid");
            let id_high =
                Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").expect("valid uuid");

            let high = Item::with_required_fields(id_high, "High");
            let low = Item::with_required_fields(id_low, "Low");
            let doc = InventoryDoc {
                version: 1,
                items: vec![high, low],
            };

            save_inventory_atomic(&db_path, &doc).expect("save should succeed");

            let raw = fs::read_to_string(&db_path).expect("saved file should be readable");
            let pos_low = raw.find(&id_low.to_string()).expect("low id should exist");
            let pos_high = raw
                .find(&id_high.to_string())
                .expect("high id should exist");
            assert!(pos_low < pos_high, "low uuid must appear before high uuid");

            let loaded = load_inventory(&db_path).expect("load should succeed");
            assert_eq!(loaded.items[0].id, id_low);
            assert_eq!(loaded.items[1].id, id_high);
        }

        #[test]
        fn save_replaces_existing_inventory_file_contents() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("inventory.json");
            fs::write(&db_path, "{\n  \"version\": 1,\n  \"items\": []\n}\n")
                .expect("seed inventory file should be writable");

            let item = Item::with_required_fields(
                Uuid::parse_str("22222222-2222-2222-2222-222222222222").expect("valid uuid"),
                "Multimeter",
            );
            let doc = InventoryDoc {
                version: 1,
                items: vec![item],
            };

            save_inventory_atomic(&db_path, &doc).expect("save should replace existing file");

            let loaded = load_inventory(&db_path).expect("updated inventory should load");
            assert_eq!(loaded.items.len(), 1);
            assert_eq!(loaded.items[0].name, "Multimeter");
        }

        #[test]
        fn save_reports_create_temp_error_when_parent_directory_is_missing() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("missing").join("inventory.json");
            let doc = InventoryDoc::default();

            let error = save_inventory_atomic(&db_path, &doc).expect_err("save must fail");
            match error {
                AppError::Io { action, source, .. } => {
                    assert_eq!(action, "create temporary inventory file");
                    assert_eq!(source.kind(), ErrorKind::NotFound);
                }
                other => panic!("expected io error, got {other}"),
            }
        }

        #[test]
        fn save_cleans_up_temp_file_when_rename_fails() {
            let dir = tempdir().expect("tempdir must be created");
            let db_path = dir.path().join("inventory.json");
            fs::create_dir(&db_path).expect("destination directory should be created");
            let doc = InventoryDoc::default();

            let error = save_inventory_atomic(&db_path, &doc).expect_err("save must fail");
            match error {
                AppError::Io { action, source, .. } => {
                    assert_eq!(action, "rename temporary inventory file into place");
                    assert_eq!(source.kind(), ErrorKind::IsADirectory);
                }
                other => panic!("expected io error, got {other}"),
            }

            let temp_entries = fs::read_dir(dir.path())
                .expect("tempdir should be readable")
                .filter_map(std::result::Result::ok)
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".inventory.json.tmp.")
                })
                .count();
            assert_eq!(temp_entries, 0, "temporary file should be removed");

            assert!(db_path.is_dir(), "destination should remain a directory");
        }
    }
}
