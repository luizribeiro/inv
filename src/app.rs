use std::path::Path;

use crate::cli::{Cli, Commands};
use crate::commands;
use crate::config;
use crate::error::Result;

pub fn run(cli: Cli) -> Result<()> {
    run_with(cli, commands::run)
}

fn run_with<F>(cli: Cli, mut runner: F) -> Result<()>
where
    F: FnMut(&Commands, &Path) -> Result<()>,
{
    let Cli { db_path, command } = cli;
    let db_path = config::resolve_db_path(db_path);
    runner(&command, &db_path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::cli::Commands;

    #[test]
    fn run_resolves_db_path_preferring_cli_override() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var("INV_DB_PATH", "from-env.json");

        let cli = Cli {
            db_path: Some(PathBuf::from("from-cli.json")),
            command: Commands::Init,
        };

        let mut seen_path = None;
        run_with(cli, |_, db_path| {
            seen_path = Some(db_path.to_path_buf());
            Ok(())
        })
        .expect("run should succeed with test runner");

        assert_eq!(seen_path, Some(PathBuf::from("from-cli.json")));
        std::env::remove_var("INV_DB_PATH");
    }

    #[test]
    fn run_resolves_db_path_from_env_when_cli_override_missing() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var("INV_DB_PATH", "from-env.json");

        let cli = Cli {
            db_path: None,
            command: Commands::Init,
        };

        let mut seen_path = None;
        run_with(cli, |_, db_path| {
            seen_path = Some(db_path.to_path_buf());
            Ok(())
        })
        .expect("run should succeed with test runner");

        assert_eq!(seen_path, Some(PathBuf::from("from-env.json")));
        std::env::remove_var("INV_DB_PATH");
    }

    #[test]
    fn run_resolves_db_path_to_default_when_no_overrides() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::remove_var("INV_DB_PATH");

        let cli = Cli {
            db_path: None,
            command: Commands::Init,
        };

        let mut seen_path = None;
        run_with(cli, |_, db_path| {
            seen_path = Some(db_path.to_path_buf());
            Ok(())
        })
        .expect("run should succeed with test runner");

        assert_eq!(seen_path, Some(PathBuf::from("./inventory.json")));
    }
}
