use std::fs;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use tempfile::tempdir;

fn inv_command() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_inv"))
}

#[test]
fn init_creates_default_inventory_json() {
    let temp = tempdir().expect("tempdir should be created");
    let db_path = temp.path().join("inventory.json");

    inv_command()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success();

    assert!(db_path.exists(), "inventory.json should be created");

    let raw = fs::read_to_string(&db_path).expect("inventory.json should be readable");
    let value: serde_json::Value =
        serde_json::from_str(&raw).expect("inventory.json should be valid json");

    assert_eq!(value["version"], 1);
    assert_eq!(value["items"], serde_json::json!([]));
}

#[test]
fn init_fails_when_inventory_file_already_exists() {
    let temp = tempdir().expect("tempdir should be created");
    let db_path = temp.path().join("inventory.json");
    fs::write(&db_path, "{\n  \"version\": 1,\n  \"items\": []\n}\n")
        .expect("seed inventory file should be written");

    inv_command()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    let raw = fs::read_to_string(&db_path).expect("inventory.json should stay readable");
    assert_eq!(raw, "{\n  \"version\": 1,\n  \"items\": []\n}\n");
}

#[test]
fn init_honors_db_path_override() {
    let temp = tempdir().expect("tempdir should be created");
    let custom_path = temp.path().join("custom-inventory.json");

    inv_command()
        .current_dir(temp.path())
        .args([
            "--db-path",
            custom_path
                .to_str()
                .expect("custom db path should be valid utf-8"),
            "init",
        ])
        .assert()
        .success();

    assert!(
        custom_path.exists(),
        "custom inventory file should be created"
    );
    assert!(
        !temp.path().join("inventory.json").exists(),
        "default inventory path should not be used"
    );
}
