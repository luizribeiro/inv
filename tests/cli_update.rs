use std::fs;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use tempfile::tempdir;

const UPDATE_TEST_INPUT_ENV: &str = "INV_UPDATE_TEST_INPUT";

fn inv_command() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_inv"))
}

fn write_inventory(temp_dir: &tempfile::TempDir, json: &str) {
    let db_path = temp_dir.path().join("inventory.json");
    fs::write(&db_path, json).expect("inventory file should be written");
}

#[test]
fn update_persists_changes_using_test_input_hook() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\",\n      \"description\": \"Old description\",\n      \"quantity\": 4,\n      \"unit\": \"pcs\",\n      \"location\": \"Shelf A\",\n      \"created_at\": \"2024-01-01T00:00:00Z\",\n      \"updated_at\": \"2024-01-01T00:00:00Z\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .env(
            UPDATE_TEST_INPUT_ENV,
            r#"{"name":"  Updated resistor pack  ","description":null,"quantity":12,"unit":" packs ","location":" Drawer C ","tags":[" passives ",""],"notes":"  updated note  "}"#,
        )
        .args(["update", "11111111-1111-1111-1111-111111111111"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated item 11111111-1111-1111-1111-111111111111"))
        .stdout(predicate::str::contains("Updated resistor pack"));

    let raw = fs::read_to_string(temp.path().join("inventory.json"))
        .expect("inventory file should be readable after update");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("inventory should be json");

    let item = &value["items"][0];
    assert_eq!(item["name"], "Updated resistor pack");
    assert_eq!(item["description"], serde_json::Value::Null);
    assert_eq!(item["quantity"], 12);
    assert_eq!(item["unit"], "packs");
    assert_eq!(item["location"], "Drawer C");
    assert_eq!(item["tags"], serde_json::json!(["passives"]));
    assert_eq!(item["notes"], "updated note");
    assert_eq!(item["created_at"], "2024-01-01T00:00:00Z");

    let updated_at = item["updated_at"]
        .as_str()
        .expect("updated_at should be a string timestamp");
    assert_ne!(updated_at, "2024-01-01T00:00:00Z");
}

#[test]
fn update_fails_when_item_is_not_found() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .env(UPDATE_TEST_INPUT_ENV, r#"{"name":"New name"}"#)
        .args(["update", "22222222-2222-2222-2222-222222222222"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "item '22222222-2222-2222-2222-222222222222' not found",
        ));
}

#[test]
fn update_fails_for_invalid_uuid() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": []\n}\n");

    inv_command()
        .current_dir(temp.path())
        .env(UPDATE_TEST_INPUT_ENV, r#"{"name":"New name"}"#)
        .args(["update", "not-a-uuid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid item id 'not-a-uuid' (expected UUID)",
        ));
}

#[test]
fn update_fails_when_test_input_hook_is_invalid_json() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .env(UPDATE_TEST_INPUT_ENV, "{not-json")
        .args(["update", "11111111-1111-1111-1111-111111111111"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "failed to parse INV_UPDATE_TEST_INPUT",
        ));
}

#[test]
fn update_fails_semantic_validation_and_keeps_inventory_unchanged() {
    let temp = tempdir().expect("tempdir should be created");
    let initial = "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\",\n      \"source_url\": \"https://example.com/resistors\",\n      \"updated_at\": \"2024-01-01T00:00:00Z\"\n    }\n  ]\n}\n";
    write_inventory(&temp, initial);

    inv_command()
        .current_dir(temp.path())
        .env(
            UPDATE_TEST_INPUT_ENV,
            r#"{"source_url":"ftp://example.com/invalid"}"#,
        )
        .args(["update", "11111111-1111-1111-1111-111111111111"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("scheme must be http or https"));

    let after = fs::read_to_string(temp.path().join("inventory.json"))
        .expect("inventory file should still be readable");
    assert_eq!(after, initial);
}
