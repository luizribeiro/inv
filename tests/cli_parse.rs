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
fn parses_remaining_stub_subcommands_and_returns_not_implemented() {
    let cases: &[(&[&str], &str)] = &[
        (&["label", "abc"], "label"),
        (&["label", "abc", "--json"], "label"),
        (&["ios-setup"], "ios-setup"),
        (&["ios-setup", "--url", "https://example.com"], "ios-setup"),
    ];

    for (args, expected_command) in cases {
        inv_command()
            .args(*args)
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "command '{expected_command}' is not implemented yet"
            )));
    }
}
