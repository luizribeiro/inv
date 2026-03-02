use std::fs;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use tempfile::tempdir;

fn inv_command() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_inv"))
}

fn write_inventory(temp_dir: &tempfile::TempDir, json: &str) {
    let db_path = temp_dir.path().join("inventory.json");
    fs::write(&db_path, json).expect("inventory file should be written");
}

#[test]
fn list_human_output_includes_expected_row_format() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\",\n      \"quantity\": 10\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "11111111-1111-1111-1111-111111111111\tResistor pack\t10 pcs",
        ));
}

#[test]
fn list_human_output_for_empty_inventory_shows_friendly_message() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": []\n}\n");

    inv_command()
        .current_dir(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No items found."));
}

#[test]
fn list_json_output_is_valid_array_of_items() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"22222222-2222-2222-2222-222222222222\",\n      \"name\": \"Capacitor kit\"\n    }\n  ]\n}\n",
    );

    let output = inv_command()
        .current_dir(temp.path())
        .args(["list", "--json"])
        .output()
        .expect("list --json should run");

    assert!(output.status.success(), "list --json should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid json");

    let items = value.as_array().expect("json output should be an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "22222222-2222-2222-2222-222222222222");
    assert_eq!(items[0]["name"], "Capacitor kit");
}

#[test]
fn list_json_output_for_empty_inventory_is_empty_array() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": []\n}\n");

    let output = inv_command()
        .current_dir(temp.path())
        .args(["list", "--json"])
        .output()
        .expect("list --json should run");

    assert!(output.status.success(), "list --json should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid json");

    assert_eq!(value, serde_json::json!([]));
}

#[test]
fn list_fails_when_inventory_file_is_missing() {
    let temp = tempdir().expect("tempdir should be created");

    inv_command()
        .current_dir(temp.path())
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read inventory file"));
}
