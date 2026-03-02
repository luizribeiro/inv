use std::path::PathBuf;

use url::Url;

use crate::error::{AppError, Result};

const DEFAULT_DB_PATH: &str = "./inventory.json";
const INV_DB_PATH_ENV: &str = "INV_DB_PATH";
const DEFAULT_IOS_SHORTCUT_URL: &str =
    "https://www.icloud.com/shortcuts/00000000000000000000000000000000";
const INV_IOS_SHORTCUT_URL_ENV: &str = "INV_IOS_SHORTCUT_URL";

pub fn resolve_db_path(cli_override: Option<PathBuf>) -> PathBuf {
    cli_override
        .or_else(|| std::env::var_os(INV_DB_PATH_ENV).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DB_PATH))
}

pub fn resolve_ios_shortcut_url(cli_override: Option<String>) -> Result<String> {
    let url = cli_override
        .or_else(|| std::env::var(INV_IOS_SHORTCUT_URL_ENV).ok())
        .unwrap_or_else(|| DEFAULT_IOS_SHORTCUT_URL.to_string());

    validate_https_url(&url)?;

    Ok(url)
}

fn validate_https_url(value: &str) -> Result<()> {
    let parsed = Url::parse(value).map_err(|_| AppError::InvalidUrl {
        source: value.to_string(),
        reason: "must be a valid absolute URL",
    })?;

    if parsed.scheme() != "https" {
        return Err(AppError::InvalidUrl {
            source: value.to_string(),
            reason: "URL scheme must be https",
        });
    }

    Ok(())
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

    #[test]
    fn resolve_ios_shortcut_url_prefers_cli_override() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(INV_IOS_SHORTCUT_URL_ENV, "https://example.com/from-env");

        let resolved = resolve_ios_shortcut_url(Some("https://example.com/from-cli".to_string()))
            .expect("cli URL should be valid");

        assert_eq!(resolved, "https://example.com/from-cli");
        std::env::remove_var(INV_IOS_SHORTCUT_URL_ENV);
    }

    #[test]
    fn resolve_ios_shortcut_url_falls_back_to_env_var() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(INV_IOS_SHORTCUT_URL_ENV, "https://example.com/from-env");

        let resolved = resolve_ios_shortcut_url(None).expect("env URL should be valid");

        assert_eq!(resolved, "https://example.com/from-env");
        std::env::remove_var(INV_IOS_SHORTCUT_URL_ENV);
    }

    #[test]
    fn resolve_ios_shortcut_url_uses_default_when_no_overrides() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::remove_var(INV_IOS_SHORTCUT_URL_ENV);

        let resolved = resolve_ios_shortcut_url(None).expect("default URL should be valid");

        assert_eq!(resolved, DEFAULT_IOS_SHORTCUT_URL);
    }

    #[test]
    fn resolve_ios_shortcut_url_rejects_malformed_url() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::remove_var(INV_IOS_SHORTCUT_URL_ENV);

        let error = resolve_ios_shortcut_url(Some("not a url".to_string()))
            .expect_err("malformed URL must fail");

        assert!(matches!(error, AppError::InvalidUrl { .. }));
    }

    #[test]
    fn resolve_ios_shortcut_url_rejects_non_https_url() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::remove_var(INV_IOS_SHORTCUT_URL_ENV);

        let error = resolve_ios_shortcut_url(Some("http://example.com".to_string()))
            .expect_err("non-https URL must fail");

        assert!(matches!(error, AppError::InvalidUrl { .. }));
    }

    #[test]
    fn resolve_ios_shortcut_url_rejects_malformed_env_url() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(INV_IOS_SHORTCUT_URL_ENV, "not a url");

        let error =
            resolve_ios_shortcut_url(None).expect_err("malformed env URL must fail validation");

        assert!(matches!(error, AppError::InvalidUrl { .. }));
        std::env::remove_var(INV_IOS_SHORTCUT_URL_ENV);
    }

    #[test]
    fn resolve_ios_shortcut_url_rejects_non_https_env_url() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var(INV_IOS_SHORTCUT_URL_ENV, "http://example.com/from-env");

        let error =
            resolve_ios_shortcut_url(None).expect_err("non-https env URL must fail validation");

        assert!(matches!(error, AppError::InvalidUrl { .. }));
        std::env::remove_var(INV_IOS_SHORTCUT_URL_ENV);
    }
}
