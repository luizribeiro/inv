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
fn qr_writes_png_file_for_existing_item() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    let out_path = temp.path().join("resistor-qr.png");

    inv_command()
        .current_dir(temp.path())
        .args([
            "qr",
            "11111111-1111-1111-1111-111111111111",
            "--out",
            out_path.to_string_lossy().as_ref(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Wrote QR code for item"));

    let bytes = fs::read(&out_path).expect("QR output file should be readable");
    assert!(bytes.len() > 8, "PNG file should not be empty");
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn qr_uses_default_output_path_when_out_is_not_provided() {
    let temp = tempdir().expect("tempdir should be created");
    let id = "11111111-1111-1111-1111-111111111111";
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    inv_command()
        .current_dir(temp.path())
        .args(["qr", id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Wrote QR code for item"));

    let out_path = temp.path().join(format!("{id}.png"));
    let bytes = fs::read(&out_path).expect("default QR output file should be readable");
    assert!(bytes.len() > 8, "PNG file should not be empty");
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn qr_fails_when_item_is_not_found() {
    let temp = tempdir().expect("tempdir should be created");
    write_inventory(
        &temp,
        "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n",
    );

    let out_path = temp.path().join("missing.png");

    inv_command()
        .current_dir(temp.path())
        .args([
            "qr",
            "22222222-2222-2222-2222-222222222222",
            "--out",
            out_path.to_string_lossy().as_ref(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "item '22222222-2222-2222-2222-222222222222' not found",
        ));

    assert!(
        !out_path.exists(),
        "QR file should not be created on failure"
    );
}
