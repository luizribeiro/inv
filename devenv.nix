{ pkgs, ... }:

{
  packages = [
    pkgs.git
    pkgs.ptouch-print
  ];

  languages.rust.enable = true;

  # Required with current devenv input to avoid missing default process-compose config.
  process.managers.process-compose.configFile =
    pkgs.writeText "process-compose.yaml" "version: '0.5'\nprocesses: {}\n";

  scripts.inv.exec = ''
    cargo run --quiet -- "$@"
  '';

  git-hooks.hooks = {
    cargo-fmt-check = {
      enable = true;
      entry = "cargo fmt --check";
      pass_filenames = false;
      always_run = true;
    };

    cargo-clippy = {
      enable = true;
      entry = "cargo clippy -- -D warnings";
      pass_filenames = false;
      always_run = true;
    };

    cargo-check = {
      enable = true;
      entry = "cargo check";
      pass_filenames = false;
      always_run = true;
    };

    cargo-test = {
      enable = true;
      entry = "cargo test";
      pass_filenames = false;
      always_run = true;
    };
  };
}
