use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;
use uuid::Uuid;

const ADD_TEST_INPUT_ENV: &str = "INV_ADD_TEST_INPUT";

fn inv_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_inv"))
}

fn write_inventory(temp_dir: &tempfile::TempDir, json: &str) {
    let db_path = temp_dir.path().join("inventory.json");
    fs::write(&db_path, json).expect("inventory file should be written");
}

#[test]
fn add_persists_new_item_with_defaults_using_test_input_hook() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": []\n}\n");

    inv_command()
        .current_dir(temp.path())
        .env(ADD_TEST_INPUT_ENV, r#"{"name":"Resistor pack"}"#)
        .arg("add")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added item"))
        .stdout(predicate::str::contains("Resistor pack"));

    let raw = fs::read_to_string(temp.path().join("inventory.json"))
        .expect("inventory file should be readable after add");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("inventory should be json");

    let items = value["items"].as_array().expect("items should be an array");
    assert_eq!(items.len(), 1);

    let item = &items[0];
    let id = item["id"].as_str().expect("id should be a string");
    Uuid::parse_str(id).expect("id should be a valid uuid");
    assert_eq!(item["name"], "Resistor pack");
    assert_eq!(item["quantity"], 0);
    assert_eq!(item["unit"], "pcs");
}

#[test]
fn add_persists_explicit_fields_from_test_input_hook() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": []\n}\n");

    inv_command()
        .current_dir(temp.path())
        .env(
            ADD_TEST_INPUT_ENV,
            r#"{"name":"  Capacitor kit  ","description":"  Ceramic  ","quantity":7,"unit":" packs ","location":" Shelf B ","source_url":" https://example.com/caps ","tags":[" passive ",""],"notes":"  restock  "}"#,
        )
        .arg("add")
        .assert()
        .success();

    let raw = fs::read_to_string(temp.path().join("inventory.json"))
        .expect("inventory file should be readable after add");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("inventory should be json");

    let item = &value["items"][0];
    assert_eq!(item["name"], "Capacitor kit");
    assert_eq!(item["description"], "Ceramic");
    assert_eq!(item["quantity"], 7);
    assert_eq!(item["unit"], "packs");
    assert_eq!(item["location"], "Shelf B");
    assert_eq!(item["source_url"], "https://example.com/caps");
    assert_eq!(item["tags"], serde_json::json!(["passive"]));
    assert_eq!(item["notes"], "restock");
}

#[test]
fn add_fails_when_inventory_file_is_missing() {
    let temp = tempdir().expect("tempdir should be created");

    inv_command()
        .current_dir(temp.path())
        .env(ADD_TEST_INPUT_ENV, r#"{"name":"Resistor pack"}"#)
        .arg("add")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read inventory file"));
}

#[test]
fn add_fails_when_test_input_hook_is_invalid_json() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": []\n}\n");

    inv_command()
        .current_dir(temp.path())
        .env(ADD_TEST_INPUT_ENV, "{not-json")
        .arg("add")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "failed to parse INV_ADD_TEST_INPUT",
        ));
}

#[test]
fn add_accepts_non_interactive_stdin_json_input() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": []\n}\n");

    inv_command()
        .current_dir(temp.path())
        .args(["add", "--stdin-json"])
        .write_stdin(
            r#"{"name":"Switchcraft Jack","quantity":6,"unit":"pcs","tags":["guitar","jack"]}"#,
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("Added item"))
        .stdout(predicate::str::contains("Switchcraft Jack"));

    let raw = fs::read_to_string(temp.path().join("inventory.json"))
        .expect("inventory file should be readable after add");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("inventory should be json");
    let item = &value["items"][0];
    assert_eq!(item["name"], "Switchcraft Jack");
    assert_eq!(item["quantity"], 6);
    assert_eq!(item["tags"], serde_json::json!(["guitar", "jack"]));
}

#[test]
fn add_stdin_json_rejects_invalid_json_payload() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": []\n}\n");

    inv_command()
        .current_dir(temp.path())
        .args(["add", "--stdin-json"])
        .write_stdin("{not-json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse stdin JSON input"));
}

#[test]
fn add_fails_semantic_validation_and_keeps_inventory_unchanged() {
    let temp = tempdir().expect("tempdir should be created");
    let initial = "{\n  \"version\": 1,\n  \"items\": []\n}\n";
    write_inventory(&temp, initial);

    inv_command()
        .current_dir(temp.path())
        .env(
            ADD_TEST_INPUT_ENV,
            r#"{"name":"Bad URL item","source_url":"ftp://example.com/item"}"#,
        )
        .arg("add")
        .assert()
        .failure()
        .stderr(predicate::str::contains("scheme must be http or https"));

    let after = fs::read_to_string(temp.path().join("inventory.json"))
        .expect("inventory file should still be readable");
    assert_eq!(after, initial);
}
