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
fn parses_all_subcommands_and_returns_not_implemented() {
    let cases: &[(&[&str], &str)] = &[
        (&["add"], "add"),
        (&["update", "abc"], "update"),
        (&["remove", "abc", "--yes"], "remove"),
        (&["qr", "abc"], "qr"),
        (&["qr", "abc", "--out", "qr.png"], "qr"),
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
