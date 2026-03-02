use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use tempfile::tempdir;

fn inv_command() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_inv"))
}

#[test]
fn ios_setup_prints_instructions_url_and_terminal_qr() {
    let temp = tempdir().expect("tempdir should be created");
    let url = "https://example.com/shortcut";

    inv_command()
        .current_dir(temp.path())
        .args(["ios-setup", "--url", url])
        .assert()
        .success()
        .stdout(predicate::str::contains("iOS Shortcut Setup"))
        .stdout(predicate::str::contains(format!("Shortcut URL: {url}")))
        .stdout(predicate::str::contains("█").or(predicate::str::contains("▀")));
}

#[test]
fn ios_setup_rejects_non_https_url() {
    let temp = tempdir().expect("tempdir should be created");

    inv_command()
        .current_dir(temp.path())
        .args(["ios-setup", "--url", "http://example.com/shortcut"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL scheme must be https"));
}
