{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
  packages = [ pkgs.git ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/processes/
  # processes.dev.exec = "${lib.getExe pkgs.watchexec} -n -- ls -la";
  process.managers.process-compose.configFile = pkgs.writeText "process-compose.yaml" "version: '0.5'\nprocesses: {}\n";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  scripts.inv.exec = ''
    if [ ! -f Cargo.toml ]; then
      echo "Error: Cargo.toml not found in current directory." >&2
      exit 1
    fi

    cargo run --quiet -- "$@"
  '';

  # https://devenv.sh/basics/
  enterShell = ''
    hello         # Run scripts directly
    git --version # Use packages
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
    rustc --version
    cargo --version
    cargo fmt --version
    cargo clippy --version
  '';

  # https://devenv.sh/git-hooks/
  git-hooks.hooks = {
    cargo-fmt-check = {
      enable = true;
      name = "cargo fmt --check";
      entry = "bash -c 'test -f Cargo.toml || exit 0; cargo fmt --check'";
      pass_filenames = false;
      always_run = true;
    };

    cargo-clippy = {
      enable = true;
      name = "cargo clippy -- -D warnings";
      entry = "bash -c 'test -f Cargo.toml || exit 0; cargo clippy -- -D warnings'";
      pass_filenames = false;
      always_run = true;
    };

    cargo-check = {
      enable = true;
      name = "cargo check";
      entry = "bash -c 'test -f Cargo.toml || exit 0; cargo check'";
      pass_filenames = false;
      always_run = true;
    };
  };

  # See full reference at https://devenv.sh/reference/options/
}
