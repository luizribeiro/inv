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
fn search_matches_by_name_case_insensitively() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\",\n      \"quantity\": 10\n    },\n    {\n      \"id\": \"22222222-2222-2222-2222-222222222222\",\n      \"name\": \"Capacitor kit\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["search", "RESISTOR"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "11111111-1111-1111-1111-111111111111\tResistor pack\t10 pcs",
        ))
        .stdout(predicate::str::contains("Capacitor kit").not());
}

#[test]
fn search_matches_by_description_case_insensitively() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Box\",\n      \"description\": \"Contains assorted SMD resistors\"\n    },\n    {\n      \"id\": \"22222222-2222-2222-2222-222222222222\",\n      \"name\": \"Capacitor kit\",\n      \"description\": \"Electrolytics\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["search", "smd"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "11111111-1111-1111-1111-111111111111\tBox\t0 pcs",
        ))
        .stdout(predicate::str::contains("Capacitor kit").not());
}

#[test]
fn search_no_match_returns_success_with_friendly_message() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["search", "inductor"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No items matched query 'inductor'.",
        ));
}

#[test]
fn search_json_output_shape_is_array_of_matching_items() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\",\n      \"description\": \"Through-hole\"\n    },\n    {\n      \"id\": \"22222222-2222-2222-2222-222222222222\",\n      \"name\": \"Capacitor kit\",\n      \"description\": \"Ceramic\"\n    }\n  ]\n}\n",
    );

    let output = inv_command()
        .current_dir(temp.path())
        .args(["search", "resistor", "--json"])
        .output()
        .expect("search --json should run");

    assert!(output.status.success(), "search --json should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid json");

    let items = value.as_array().expect("json output should be an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], "11111111-1111-1111-1111-111111111111");
    assert_eq!(items[0]["name"], "Resistor pack");
}

#[test]
fn search_json_output_for_no_matches_is_empty_array() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    let output = inv_command()
        .current_dir(temp.path())
        .args(["search", "inductor", "--json"])
        .output()
        .expect("search --json should run");

    assert!(output.status.success(), "search --json should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid json");

    assert_eq!(value, serde_json::json!([]));
}

#[test]
fn search_fails_when_inventory_file_is_missing() {
    let temp = tempdir().expect("tempdir should be created");

    inv_command()
        .current_dir(temp.path())
        .args(["search", "resistor"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read inventory file"));
}

#[test]
fn search_fails_for_malformed_inventory_file() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(&temp, "{\n  \"version\": 1,\n  \"items\": [\n");

    inv_command()
        .current_dir(temp.path())
        .args(["search", "resistor"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse JSON from"));
}
