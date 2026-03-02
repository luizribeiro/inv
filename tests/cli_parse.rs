use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;

fn inv_command() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_inv"))
}

#[test]
fn help_smoke_test() {
    inv_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn remove_subcommand_is_wired_and_no_longer_reports_not_implemented() {
    inv_command()
        .args(["remove", "abc", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid item id 'abc' (expected UUID)",
        ))
        .stderr(predicate::str::contains("not implemented").not());
}

#[test]
fn qr_subcommand_is_wired_and_no_longer_reports_not_implemented() {
    inv_command()
        .args(["qr", "abc"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid item id 'abc' (expected UUID)",
        ))
        .stderr(predicate::str::contains("not implemented").not());
}

#[test]
fn label_subcommand_is_wired_and_no_longer_reports_not_implemented() {
    inv_command()
        .args(["label", "abc"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid item id 'abc' (expected UUID)",
        ))
        .stderr(predicate::str::contains("not implemented").not());
}

#[test]
fn ios_setup_subcommand_is_wired_and_no_longer_reports_not_implemented() {
    inv_command()
        .args(["ios-setup", "--url", "https://example.com"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Shortcut URL: https://example.com",
        ))
        .stderr(predicate::str::contains("not implemented").not());
}

#[test]
fn add_help_mentions_stdin_json_flag() {
    inv_command()
        .args(["add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--stdin-json"));
}
