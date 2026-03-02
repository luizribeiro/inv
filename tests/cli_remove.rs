use std::fs;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use tempfile::tempdir;

const FORCE_NON_INTERACTIVE_ENV: &str = "INV_FORCE_NON_INTERACTIVE";

fn inv_command() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_inv"))
}

fn write_inventory(temp_dir: &tempfile::TempDir, json: &str) {
    let db_path = temp_dir.path().join("inventory.json");
    fs::write(&db_path, json).expect("inventory file should be written");
}

#[test]
fn remove_with_yes_removes_matching_item() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    },\n    {\n      \"id\": \"22222222-2222-2222-2222-222222222222\",\n      \"name\": \"Capacitor kit\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["remove", "11111111-1111-1111-1111-111111111111", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Removed item 11111111-1111-1111-1111-111111111111",
        ));

    let raw = fs::read_to_string(temp.path().join("inventory.json"))
        .expect("inventory file should be readable after remove");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("inventory should be json");

    let items = value["items"].as_array().expect("items should be an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "22222222-2222-2222-2222-222222222222");
}

#[test]
fn remove_fails_when_item_is_not_found() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["remove", "22222222-2222-2222-2222-222222222222", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "item '22222222-2222-2222-2222-222222222222' not found",
        ));
}

#[test]
fn remove_with_yes_bypasses_non_interactive_guard() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .env(FORCE_NON_INTERACTIVE_ENV, "1")
        .args(["remove", "11111111-1111-1111-1111-111111111111", "--yes"])
        .assert()
        .success();

    let raw = fs::read_to_string(temp.path().join("inventory.json"))
        .expect("inventory file should be readable after remove");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("inventory should be json");
    let items = value["items"].as_array().expect("items should be an array");
    assert!(items.is_empty());
}

#[test]
fn remove_requires_yes_in_non_interactive_mode() {
    let temp = tempdir().expect("tempdir should be created");
    let initial = "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n";
    write_inventory(&temp, initial);

    inv_command()
        .current_dir(temp.path())
        .env(FORCE_NON_INTERACTIVE_ENV, "1")
        .args(["remove", "11111111-1111-1111-1111-111111111111"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "refusing to remove item in non-interactive mode without --yes",
        ));

    let after = fs::read_to_string(temp.path().join("inventory.json"))
        .expect("inventory file should still be readable");
    assert_eq!(after, initial);
}
