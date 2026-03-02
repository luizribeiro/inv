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
fn label_human_output_prints_placeholder_spec() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\",\n      \"quantity\": 10,\n      \"location\": \"Drawer A\",\n      \"bin_size\": \"small\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["label", "11111111-1111-1111-1111-111111111111"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Label placeholder (v1)"))
        .stdout(predicate::str::contains(
            "id: 11111111-1111-1111-1111-111111111111",
        ))
        .stdout(predicate::str::contains("name: Resistor pack"))
        .stdout(predicate::str::contains("quantity: 10 pcs"))
        .stdout(predicate::str::contains("location: Drawer A"))
        .stdout(predicate::str::contains("bin_size: small"));
}

#[test]
fn label_human_output_uses_not_set_for_missing_optional_fields() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"22222222-2222-2222-2222-222222222222\",\n      \"name\": \"Capacitor kit\",\n      \"quantity\": 3\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["label", "22222222-2222-2222-2222-222222222222"])
        .assert()
        .success()
        .stdout(predicate::str::contains("location: (not set)"))
        .stdout(predicate::str::contains("bin_size: (not set)"));
}

#[test]
fn label_json_output_matches_expected_shape() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"22222222-2222-2222-2222-222222222222\",\n      \"name\": \"Capacitor kit\",\n      \"quantity\": 3\n    }\n  ]\n}\n",
    );

    let output = inv_command()
        .current_dir(temp.path())
        .args(["label", "22222222-2222-2222-2222-222222222222", "--json"])
        .output()
        .expect("label --json should run");

    assert!(output.status.success(), "label --json should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid json");

    assert_eq!(value["id"], "22222222-2222-2222-2222-222222222222");
    assert_eq!(value["name"], "Capacitor kit");
    assert_eq!(value["quantity"], 3);
    assert_eq!(value["unit"], "pcs");
    assert!(value.get("location").is_none());
    assert!(value.get("bin_size").is_none());
}

#[test]
fn label_fails_when_item_is_not_found() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["label", "33333333-3333-3333-3333-333333333333"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "item '33333333-3333-3333-3333-333333333333' not found",
        ));
}
