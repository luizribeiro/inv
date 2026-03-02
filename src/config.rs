use std::path::PathBuf;

const DEFAULT_DB_PATH: &str = "./inventory.json";
const INV_DB_PATH_ENV: &str = "INV_DB_PATH";

pub fn resolve_db_path(cli_override: Option<PathBuf>) -> PathBuf {
    cli_override
        .or_else(|| std::env::var_os(INV_DB_PATH_ENV).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DB_PATH))
}

#[cfg(test)]
pub(crate) fn env_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_db_path_prefers_cli_override() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(INV_DB_PATH_ENV, "from-env.json");

        let resolved = resolve_db_path(Some(PathBuf::from("from-cli.json")));

        assert_eq!(resolved, PathBuf::from("from-cli.json"));
        std::env::remove_var(INV_DB_PATH_ENV);
    }

    #[test]
    fn resolve_db_path_falls_back_to_env_var() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(INV_DB_PATH_ENV, "from-env.json");

        let resolved = resolve_db_path(None);

        assert_eq!(resolved, PathBuf::from("from-env.json"));
        std::env::remove_var(INV_DB_PATH_ENV);
    }

    #[test]
    fn resolve_db_path_uses_default_when_no_overrides() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::remove_var(INV_DB_PATH_ENV);

        let resolved = resolve_db_path(None);

        assert_eq!(resolved, PathBuf::from("./inventory.json"));
    }
}
