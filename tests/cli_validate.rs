use std::fs;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use tempfile::tempdir;

fn inv_command() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_inv"))
}

#[test]
fn validate_passes_for_valid_inventory_file() {
    let temp = tempdir().expect("tempdir should be created");
    let db_path = temp.path().join("inventory.json");
    fs::write(&db_path, "{\n  \"version\": 1,\n  \"items\": []\n}\n")
        .expect("inventory file should be written");

    inv_command()
        .current_dir(temp.path())
        .arg("validate")
        .assert()
        .success();
}

#[test]
fn validate_fails_when_inventory_file_is_missing() {
    let temp = tempdir().expect("tempdir should be created");

    inv_command()
        .current_dir(temp.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read inventory file"));
}

#[test]
fn validate_fails_for_schema_invalid_inventory_file() {
    let temp = tempdir().expect("tempdir should be created");
    let db_path = temp.path().join("inventory.json");
    fs::write(
        &db_path,
        "{\n  \"version\": 1,\n  \"items\": [{\n    \"id\": \"11111111-1111-1111-1111-111111111111\"\n  }]\n}\n",
    )
    .expect("inventory file should be written");

    inv_command()
        .current_dir(temp.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema validation failed"));
}

#[test]
fn validate_fails_for_semantic_invalid_inventory_file() {
    let temp = tempdir().expect("tempdir should be created");
    let db_path = temp.path().join("inventory.json");
    fs::write(
        &db_path,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"First\"\n    },\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Second\"\n    }\n  ]\n}\n",
    )
    .expect("inventory file should be written");

    inv_command()
        .current_dir(temp.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicate item id"));
}
