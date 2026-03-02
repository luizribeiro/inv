use std::io::IsTerminal;
use std::path::Path;

use dialoguer::Confirm;
use uuid::Uuid;

use crate::error::{AppError, Result};

const FORCE_NON_INTERACTIVE_ENV: &str = "INV_FORCE_NON_INTERACTIVE";
const FORCE_INTERACTIVE_ENV: &str = "INV_FORCE_INTERACTIVE";

pub fn run(db_path: &Path, id: &str, yes: bool) -> Result<()> {
    run_with_confirm(db_path, id, yes, |prompt| {
        Confirm::new()
            .with_prompt(prompt.to_string())
            .default(false)
            .interact()
            .map_err(|error| {
                AppError::Validation(format!("failed to read removal confirmation: {error}"))
            })
    })
}

fn run_with_confirm<F>(db_path: &Path, id: &str, yes: bool, mut confirm: F) -> Result<()>
where
    F: FnMut(&str) -> Result<bool>,
{
    let item_id = Uuid::parse_str(id)
        .map_err(|_| AppError::Validation(format!("invalid item id '{id}' (expected UUID)")))?;

    let mut doc = crate::storage::load_inventory(db_path)?;
    let item_index = doc
        .items
        .iter()
        .position(|candidate| candidate.id == item_id)
        .ok_or_else(|| AppError::Validation(format!("item '{item_id}' not found")))?;

    if !yes {
        if !is_interactive() {
            return Err(AppError::Validation(
                "refusing to remove item in non-interactive mode without --yes".to_string(),
            ));
        }

        let item = &doc.items[item_index];
        let prompt = format!("Remove item {} ({})?", item.id.as_hyphenated(), item.name);
        if !confirm(&prompt)? {
            println!("Removal aborted.");
            return Ok(());
        }
    }

    let removed = doc.items.remove(item_index);
    crate::model::validate_semantics(&doc)?;
    crate::storage::save_inventory_atomic(db_path, &doc)?;

    println!(
        "Removed item {}: {}",
        removed.id.as_hyphenated(),
        removed.name
    );

    Ok(())
}

fn is_interactive() -> bool {
    if std::env::var_os(FORCE_NON_INTERACTIVE_ENV).is_some() {
        return false;
    }

    if std::env::var_os(FORCE_INTERACTIVE_ENV).is_some() {
        return true;
    }

    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn remove_is_interactive_honors_force_non_interactive_env() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        std::env::remove_var(FORCE_INTERACTIVE_ENV);
        std::env::set_var(FORCE_NON_INTERACTIVE_ENV, "1");
        assert!(!is_interactive());
        std::env::remove_var(FORCE_NON_INTERACTIVE_ENV);
    }

    #[test]
    fn remove_run_with_confirm_false_keeps_inventory_unchanged() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::remove_var(FORCE_NON_INTERACTIVE_ENV);
        std::env::set_var(FORCE_INTERACTIVE_ENV, "1");

        let dir = tempdir().expect("tempdir should be created");
        let db_path = dir.path().join("inventory.json");
        let initial = "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    }\n  ]\n}\n";
        fs::write(&db_path, initial).expect("inventory file should be written");

        run_with_confirm(
            &db_path,
            "11111111-1111-1111-1111-111111111111",
            false,
            |_| Ok(false),
        )
        .expect("declined confirmation should still be ok");

        let after = fs::read_to_string(&db_path).expect("inventory file should be readable");
        assert_eq!(after, initial);

        std::env::remove_var(FORCE_INTERACTIVE_ENV);
    }

    #[test]
    fn remove_run_with_confirm_true_removes_item() {
        let _guard = crate::config::env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::remove_var(FORCE_NON_INTERACTIVE_ENV);
        std::env::set_var(FORCE_INTERACTIVE_ENV, "1");

        let dir = tempdir().expect("tempdir should be created");
        let db_path = dir.path().join("inventory.json");
        fs::write(
            &db_path,
            "{\n  \"version\": 1,\n  \"items\": [\n    {\n      \"id\": \"11111111-1111-1111-1111-111111111111\",\n      \"name\": \"Resistor pack\"\n    },\n    {\n      \"id\": \"22222222-2222-2222-2222-222222222222\",\n      \"name\": \"Capacitor kit\"\n    }\n  ]\n}\n",
        )
        .expect("inventory file should be written");

        run_with_confirm(
            &db_path,
            "11111111-1111-1111-1111-111111111111",
            false,
            |_| Ok(true),
        )
        .expect("accepted confirmation should remove item");

        let raw = fs::read_to_string(&db_path).expect("inventory should be readable");
        let value: serde_json::Value = serde_json::from_str(&raw).expect("inventory json");
        let items = value["items"].as_array().expect("items array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"], "22222222-2222-2222-2222-222222222222");

        std::env::remove_var(FORCE_INTERACTIVE_ENV);
    }
}
