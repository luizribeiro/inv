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
fn show_human_output_succeeds_for_existing_item() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\",\n      \"description\": \"1k ohm\",\n      \"quantity\": 10\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["show", "11111111-1111-1111-1111-111111111111"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "id: 11111111-1111-1111-1111-111111111111",
        ))
        .stdout(predicate::str::contains("name: Resistor pack"))
        .stdout(predicate::str::contains("quantity: 10 pcs"))
        .stdout(predicate::str::contains("description: 1k ohm"));
}

#[test]
fn show_fails_when_item_is_not_found() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["show", "22222222-2222-2222-2222-222222222222"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "item '22222222-2222-2222-2222-222222222222' not found",
        ));
}

#[test]
fn show_fails_for_invalid_uuid() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": []\n}\n");

    inv_command()
        .current_dir(temp.path())
        .args(["show", "not-a-uuid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid item id 'not-a-uuid' (expected UUID)",
        ));
}

#[test]
fn show_json_output_matches_selected_item() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    },\n    {\n      \"id\": \"22222222-2222-2222-2222-222222222222\",\n      \"name\": \"Capacitor kit\",\n      \"quantity\": 3\n    }\n  ]\n}\n",
    );

    let output = inv_command()
        .current_dir(temp.path())
        .args(["show", "22222222-2222-2222-2222-222222222222", "--json"])
        .output()
        .expect("show --json should run");

    assert!(output.status.success(), "show --json should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid json");

    assert_eq!(value["id"], "22222222-2222-2222-2222-222222222222");
    assert_eq!(value["name"], "Capacitor kit");
    assert_eq!(value["quantity"], 3);
}
